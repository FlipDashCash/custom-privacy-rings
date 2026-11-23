use std::thread;

use anyhow::{anyhow, ensure, Context as _, Result};
use sha2::{Digest, Sha256};
use solana_account::Account;
use solana_address::{Address, MAX_SEED_LEN};
use solana_loader_v3_interface::{instruction as loader, state::UpgradeableLoaderState};
use solana_message::VersionedMessage;
use solana_packet::PACKET_DATA_SIZE;
use solana_rpc_client::rpc_client::SerializableMessage;
use solana_signature::Signature;
use solana_system_interface::instruction as system;
use solana_transaction_error::TransactionError;
use spr_client::{ClientError, Rpc};
use zolana_operator_custody::CustodyError;

use super::{system_wallet, ProgramUpload, UploadBuffer, LOADER, SLOT_POLL};
use crate::{
    submission::{OperatorTransaction, Operators},
    transaction_error, Context,
};

struct Chunk<'a> {
    offset: usize,
    bytes: &'a [u8],
}

/// The RPC client serializes legacy and v0 messages only.
struct FeeQuote<'a>(&'a VersionedMessage);

struct UploadCost {
    fee_per_signature: u64,
    writes: usize,
    rent_due: u64,
}

#[must_use]
enum BatchEnd {
    Complete,
    Expired(anyhow::Error),
}

#[must_use]
enum Attempt {
    Landed,
    Refused(anyhow::Error),
    Transient(anyhow::Error),
}

impl<'a> ProgramUpload<'a> {
    pub(super) fn bind(self, program: Address) -> Result<UploadBuffer<'a>> {
        let digest = Sha256::new()
            .chain_update(b"zolana-ring-loader-buffer-v1")
            .chain_update(program.as_ref())
            .chain_update(self.file.binary.sha256)
            .finalize();
        let seed = hex::encode(&digest[..MAX_SEED_LEN / 2]);
        let address = Address::create_with_seed(&self.authority.address(), &seed, &LOADER)?;
        Ok(UploadBuffer {
            address,
            seed,
            source: self,
        })
    }
}

impl UploadBuffer<'_> {
    pub(super) fn fund_and_write(&self, ctx: &Context, required_rent: u64) -> Result<()> {
        let account = ctx.rpc().get_account(self.address)?;
        let contents = account
            .as_ref()
            .map(|account| self.contents(account))
            .transpose()?
            .flatten();
        let mut pending = self.pending(contents)?;
        let binary_len = self.source.file.bytes.len();
        let size = UpgradeableLoaderState::size_of_buffer(binary_len);
        let rent = ctx.rpc().get_minimum_balance_for_rent_exemption(size)?;
        let held = account
            .as_ref()
            .map_or(0, |account| account.lamports.min(rent));
        let operators = Operators::solo(self.source.authority);
        let payer = self.source.authority.address();

        let (blockhash, _) = ctx.rpc().get_latest_blockhash()?;
        let quote = OperatorTransaction {
            operators,
            instructions: &[self.write(&Chunk {
                offset: 0,
                bytes: &[],
            })?],
        }
        .message(ctx, blockhash)?;
        let fee_per_signature = ctx.rpc().client().get_fee_for_message(&FeeQuote(&quote))?;
        let required = UploadCost {
            fee_per_signature,
            writes: pending.len(),
            rent_due: required_rent.saturating_sub(held),
        }
        .lamports()?;
        // A short balance fails before the loader writes.
        ctx.fund(payer, required)?;
        if contents.is_none() {
            let mut instructions =
                loader::create_buffer(&payer, &self.address, &payer, rent, binary_len)?;
            instructions.splice(
                ..1,
                [
                    system::transfer(&payer, &self.address, rent - held),
                    system::allocate_with_seed(
                        &self.address,
                        &payer,
                        &self.seed,
                        u64::try_from(size)?,
                        &LOADER,
                    ),
                ],
            );
            ctx.send_operator(operators, &instructions)?;
        }
        // Each batch must shrink the pending set, a write is idempotent.
        while !pending.is_empty() {
            let attempted = pending.len();
            let end = self.write_batch(ctx, &pending)?;
            let uploaded = ctx
                .rpc()
                .get_account(self.address)?
                .context("upload buffer is missing")?;
            pending = self.pending(self.contents(&uploaded)?)?;
            if pending.len() >= attempted {
                let buffer = self.address;
                return Err(match end {
                    BatchEnd::Expired(error) => error.context(format!(
                        "no write reached buffer {buffer} before its blockhash expired"
                    )),
                    BatchEnd::Complete => {
                        anyhow!("upload buffer {buffer} differs from the program binary")
                    }
                });
            }
        }
        Ok(())
    }

    pub(super) fn refund(&self, ctx: &Context) -> Result<()> {
        if let Some(account) = ctx.rpc().get_account(self.address)? {
            if self.contents(&account)?.is_some() {
                let payer = self.source.authority.address();
                let close = loader::close(&self.address, &payer, &payer, false);
                ctx.send_operator(Operators::solo(self.source.authority), &[close])?;
                println!("refunded {}", self.address);
                return Ok(());
            }
        }
        println!("no loader buffer at {}", self.address);
        Ok(())
    }

    /// One blockhash signs each write, a transient failure is retried until it expires.
    fn write_batch(&self, ctx: &Context, pending: &[Chunk<'_>]) -> Result<BatchEnd> {
        let operators = Operators::solo(self.source.authority);
        let (blockhash, last_valid_block_height) = ctx.rpc().get_latest_blockhash()?;
        for chunk in pending {
            let message = OperatorTransaction {
                operators,
                instructions: &[self.write(chunk)?],
            }
            .message(ctx, blockhash)?;
            loop {
                match Attempt::of(operators.land(ctx, &message)) {
                    Attempt::Landed => break,
                    Attempt::Refused(error) => {
                        return Err(error.context(format!(
                            "write to buffer {} failed at byte {}",
                            self.address, chunk.offset
                        )))
                    }
                    Attempt::Transient(error) => {
                        if ctx.rpc().get_block_height()? > last_valid_block_height {
                            return Ok(BatchEnd::Expired(error));
                        }
                        thread::sleep(SLOT_POLL);
                    }
                }
            }
        }
        Ok(BatchEnd::Complete)
    }

    /// A zero chunk stays unwritten, the loader allocates a zeroed buffer.
    fn pending(&self, contents: Option<&[u8]>) -> Result<Vec<Chunk<'_>>> {
        let header = self
            .write(&Chunk {
                offset: 0,
                bytes: &[],
            })?
            .data
            .len();
        // The loader limits decoded instruction data to one packet.
        let chunk_len = PACKET_DATA_SIZE - header;
        let mut pending = Vec::new();
        for (index, bytes) in self.source.file.bytes.chunks(chunk_len).enumerate() {
            let offset = index * chunk_len;
            if let Some(stored) = contents.map(|stored| &stored[offset..offset + bytes.len()]) {
                if stored == bytes {
                    continue;
                }
                ensure!(
                    stored
                        .iter()
                        .zip(bytes)
                        .all(|(actual, wanted)| *actual == 0 || actual == wanted),
                    "upload buffer {} differs at byte {offset}",
                    self.address
                );
            }
            if bytes.iter().any(|byte| *byte != 0) {
                pending.push(Chunk { offset, bytes });
            }
        }
        Ok(pending)
    }

    fn write(&self, chunk: &Chunk<'_>) -> Result<solana_instruction::Instruction> {
        Ok(loader::write(
            &self.address,
            &self.source.authority.address(),
            u32::try_from(chunk.offset)?,
            chunk.bytes.to_vec(),
        ))
    }

    fn contents<'a>(&self, account: &'a Account) -> Result<Option<&'a [u8]>> {
        if system_wallet(account) {
            return Ok(None);
        }
        ensure!(
            account.owner == LOADER && !account.executable,
            "invalid upload buffer owner or executable flag"
        );
        let (header, bytes) = account
            .data
            .split_at_checked(UpgradeableLoaderState::size_of_buffer_metadata())
            .context("truncated upload buffer")?;
        let expected = UpgradeableLoaderState::Buffer {
            authority_address: Some(self.source.authority.address()),
        };
        ensure!(
            wincode::deserialize::<UpgradeableLoaderState>(header)? == expected,
            "invalid upload buffer state or authority"
        );
        ensure!(
            bytes.len() == self.source.file.bytes.len(),
            "invalid upload buffer capacity"
        );
        Ok(Some(bytes))
    }
}

impl UploadCost {
    /// One signature creates the buffer, two more deploy or extend and upgrade.
    fn lamports(self) -> Result<u64> {
        let signatures = u64::try_from(self.writes)? + 3;
        self.fee_per_signature
            .checked_mul(signatures)
            .and_then(|fees| self.rent_due.checked_add(fees))
            .context("deploy funding exceeds the lamport limit")
    }
}

impl Attempt {
    /// A repeat of a confirmed write answers `AlreadyProcessed`.
    fn of(sent: Result<Signature>) -> Self {
        let Err(error) = sent else {
            return Self::Landed;
        };
        if let Some(custody) = error.downcast_ref::<CustodyError>() {
            return match custody {
                CustodyError::Service(_) => Self::Transient(error),
                _ => Self::Refused(error),
            };
        }
        match error
            .downcast_ref::<ClientError>()
            .and_then(transaction_error)
        {
            Some(TransactionError::AlreadyProcessed) => Self::Landed,
            Some(TransactionError::BlockhashNotFound) | None => Self::Transient(error),
            Some(_) => Self::Refused(error),
        }
    }
}

impl SerializableMessage for FeeQuote<'_> {
    fn serialize(&self) -> Vec<u8> {
        self.0.serialize()
    }
}

#[cfg(test)]
mod tests {
    use solana_instruction::error::InstructionError;
    use solana_keypair::Keypair;

    use super::{
        super::{ProgramBinary, ProgramFile},
        *,
    };
    use crate::{context::transaction_failure, tests::transaction_error};

    const TAIL: usize = 10;

    struct Upload {
        authority: Keypair,
        file: ProgramFile,
    }

    impl Upload {
        fn of(bytes: Vec<u8>) -> Self {
            Self {
                authority: Keypair::new(),
                file: ProgramFile {
                    binary: ProgramBinary {
                        len: bytes.len(),
                        sha256: Sha256::digest(&bytes).into(),
                    },
                    bytes,
                },
            }
        }

        /// A nonzero chunk, a zero chunk, a nonzero chunk and a short tail.
        fn sample() -> Self {
            let chunk = chunk_len();
            let bytes = [
                vec![7; chunk],
                vec![0; chunk],
                vec![9; chunk],
                vec![5; TAIL],
            ]
            .concat();
            Self::of(bytes)
        }

        fn buffer(&self) -> UploadBuffer<'_> {
            ProgramUpload {
                authority: &self.authority,
                file: &self.file,
            }
            .bind(Address::new_from_array([3; 32]))
            .expect("bound")
        }

        fn account(&self, authority: Address, stored: &[u8]) -> Account {
            let state = UpgradeableLoaderState::Buffer {
                authority_address: Some(authority),
            };
            let mut data = wincode::serialize(&state).expect("header");
            data.extend_from_slice(stored);
            Account {
                lamports: 1,
                data,
                owner: LOADER,
                ..Account::default()
            }
        }
    }

    fn chunk_len() -> usize {
        let probe = Upload::of(vec![1; 2 * PACKET_DATA_SIZE]);
        let buffer = probe.buffer();
        let pending = buffer.pending(None).expect("pending");
        pending[0].bytes.len()
    }

    fn spans(pending: Result<Vec<Chunk<'_>>>) -> Vec<(usize, usize)> {
        pending
            .expect("pending")
            .iter()
            .map(|chunk| (chunk.offset, chunk.bytes.len()))
            .collect()
    }

    fn refusal<T>(result: Result<T>) -> String {
        result.err().expect("refused").to_string()
    }

    #[test]
    fn an_absent_buffer_needs_every_nonzero_chunk_and_a_full_chunk_fills_one_packet() {
        let upload = Upload::sample();
        let buffer = upload.buffer();
        let chunk = chunk_len();
        let pending = buffer.pending(None).expect("pending");
        assert_eq!(
            buffer.write(&pending[0]).expect("write").data.len(),
            PACKET_DATA_SIZE
        );
        assert_eq!(
            spans(Ok(pending)),
            [(0, chunk), (2 * chunk, chunk), (3 * chunk, TAIL)]
        );
    }

    #[test]
    fn a_stored_chunk_is_skipped_and_a_partial_write_is_repeated() {
        let upload = Upload::sample();
        let buffer = upload.buffer();
        let chunk = chunk_len();
        let wanted = &upload.file.bytes;
        let every = vec![(0, chunk), (2 * chunk, chunk), (3 * chunk, TAIL)];
        let zeroed = vec![0; wanted.len()];
        let mut first_written = zeroed.clone();
        first_written[..chunk].copy_from_slice(&wanted[..chunk]);
        let mut half_written = wanted.clone();
        half_written[2 * chunk + chunk / 2..3 * chunk].fill(0);
        for (stored, expected) in [
            (wanted.clone(), Vec::new()),
            (zeroed, every.clone()),
            (first_written, every[1..].to_vec()),
            (half_written, every[1..2].to_vec()),
        ] {
            assert_eq!(spans(buffer.pending(Some(&stored))), expected);
        }
    }

    #[test]
    fn a_stored_byte_that_differs_is_refused_with_its_chunk_offset() {
        let upload = Upload::sample();
        let buffer = upload.buffer();
        let chunk = chunk_len();
        for (position, offset) in [(2 * chunk + 3, 2 * chunk), (chunk + 1, chunk)] {
            let mut stored = upload.file.bytes.clone();
            stored[position] = 1;
            let refused = refusal(buffer.pending(Some(&stored)));
            assert!(
                refused.ends_with(&format!("differs at byte {offset}")),
                "{refused}"
            );
        }
    }

    #[test]
    fn only_a_loader_buffer_of_the_authority_and_the_binary_length_is_resumed() {
        let upload = Upload::sample();
        let buffer = upload.buffer();
        let authority = buffer.source.authority.address();
        let stored = &upload.file.bytes;
        let held = upload.account(authority, stored);
        assert_eq!(
            buffer.contents(&held).expect("resumed"),
            Some(stored.as_slice())
        );
        let wallet = Account {
            lamports: 1,
            ..Account::default()
        };
        assert_eq!(buffer.contents(&wallet).expect("vacant"), None);
        let truncated = Account {
            data: held.data[..4].to_vec(),
            ..held.clone()
        };
        for (account, reason) in [
            (
                Account {
                    owner: Address::new_from_array([9; 32]),
                    ..held.clone()
                },
                "invalid upload buffer owner or executable flag",
            ),
            (
                Account {
                    executable: true,
                    ..held.clone()
                },
                "invalid upload buffer owner or executable flag",
            ),
            (truncated, "truncated upload buffer"),
            (
                upload.account(Address::new_from_array([9; 32]), stored),
                "invalid upload buffer state or authority",
            ),
            (
                upload.account(authority, &stored[1..]),
                "invalid upload buffer capacity",
            ),
        ] {
            assert_eq!(refusal(buffer.contents(&account)), reason);
        }
    }

    #[test]
    fn another_program_or_binary_binds_another_buffer() {
        let upload = Upload::sample();
        let bind = |program: [u8; 32], file: &ProgramFile| {
            ProgramUpload {
                authority: &upload.authority,
                file,
            }
            .bind(Address::new_from_array(program))
            .expect("bound")
            .address
        };
        let other = Upload::of(vec![1; TAIL]).file;
        assert_eq!(bind([3; 32], &upload.file), upload.buffer().address);
        assert_ne!(bind([4; 32], &upload.file), upload.buffer().address);
        assert_ne!(bind([3; 32], &other), upload.buffer().address);
    }

    #[test]
    fn upload_funding_pays_each_write_and_three_more_signatures_over_the_rent_due() {
        let cost = |fee_per_signature, writes, rent_due| {
            UploadCost {
                fee_per_signature,
                writes,
                rent_due,
            }
            .lamports()
        };
        assert_eq!(cost(5_000, 0, 0).expect("fees only"), 15_000);
        assert_eq!(cost(5_000, 2, 1_000).expect("priced"), 26_000);
        assert_eq!(cost(u64::MAX / 4, 1, 3).expect("limit"), u64::MAX);
        for overflow in [cost(u64::MAX / 4 + 1, 1, 0), cost(u64::MAX / 4, 1, 4)] {
            assert_eq!(
                refusal(overflow),
                "deploy funding exceeds the lamport limit"
            );
        }
    }

    #[test]
    fn a_send_outcome_is_landed_retried_or_refused() {
        let cluster = |error| Err(transaction_failure(transaction_error(error)));
        let custody = |error: CustodyError| Err(anyhow::Error::from(error));
        for landed in [
            Ok(Signature::default()),
            cluster(TransactionError::AlreadyProcessed),
        ] {
            assert!(matches!(Attempt::of(landed), Attempt::Landed));
        }
        for transient in [
            cluster(TransactionError::BlockhashNotFound),
            Err(transaction_failure(ClientError::Rpc("reset".to_owned()))),
            custody(CustodyError::Service("throttled".into())),
        ] {
            assert!(matches!(Attempt::of(transient), Attempt::Transient(_)));
        }
        for refused in [
            cluster(TransactionError::InstructionError(
                0,
                InstructionError::InvalidAccountData,
            )),
            cluster(TransactionError::InsufficientFundsForFee),
            custody(CustodyError::AddressMismatch),
            custody(CustodyError::Transaction(ClientError::Rpc(
                "reset".to_owned(),
            ))),
        ] {
            assert!(matches!(Attempt::of(refused), Attempt::Refused(_)));
        }
    }
}
