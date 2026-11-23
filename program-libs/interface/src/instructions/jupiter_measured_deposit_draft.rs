//! Inert wire proposal for a single measured legacy SPL output after Jupiter.
//!
//! Only the host interface test imports this file. Neither library exports nor
//! any program register the tag. Parsing leaves the descriptor unverified and
//! grants no custody, output credit, or authority to invoke an instruction.

#![forbid(unsafe_code)]

pub const DRAFT_TAG: u8 = 68;
pub const DRAFT_VERSION: u8 = 1;

// Fields retained from the Custom dynamic descriptor V1 statement and its
// 192-byte proof envelope. The proof is not a stock transfer proof or
// a substitute for a selected descriptor verifier.
const EXPIRY: usize = 2;
const TRACKING_REVISION: usize = EXPIRY + 8;
const TRACKING_ROOT: usize = TRACKING_REVISION + 8;
const OWNER_UTXO_HASH: usize = TRACKING_ROOT + 32;
const VIEWING_KEY: usize = OWNER_UTXO_HASH + 32;
const SALT: usize = VIEWING_KEY + 33;
const RECIPIENT_TAG: usize = SALT + 16;
const ZERO_RECORD: usize = RECIPIENT_TAG + 32;
const AUDITOR_ENVELOPE: usize = ZERO_RECORD + 105;
const PROOF: usize = AUDITOR_ENVELOPE + 64;
pub const PROOF_LEN: usize = 192;
pub const FRAME_LEN: usize = PROOF + PROOF_LEN;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DraftFrame<'a> {
    pub expiry_slot: u64,
    pub tracking_revision: u64,
    pub tracking_root: &'a [u8; 32],
    pub owner_utxo_hash: &'a [u8; 32],
    pub tx_viewing_pubkey: &'a [u8; 33],
    pub salt: &'a [u8; 16],
    pub recipient_tag: &'a [u8; 32],
    pub recipient_zero_ciphertext: &'a [u8; 105],
    pub auditor_ciphertext: &'a [u8; 64],
    pub descriptor_proof_bytes: &'a [u8; PROOF_LEN],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameError {
    Length,
    Header,
}

fn read<const N: usize>(bytes: &[u8], offset: usize) -> Result<&[u8; N], FrameError> {
    bytes
        .get(offset..offset + N)
        .and_then(|field| field.try_into().ok())
        .ok_or(FrameError::Length)
}

fn write<const N: usize>(bytes: &mut [u8; FRAME_LEN], offset: usize, field: &[u8; N]) {
    bytes[offset..offset + N].copy_from_slice(field);
}

impl<'a> DraftFrame<'a> {
    /// Decode only the fixed-size envelope. No field is authenticated here.
    pub fn decode(bytes: &'a [u8]) -> Result<Self, FrameError> {
        if bytes.len() != FRAME_LEN {
            return Err(FrameError::Length);
        }
        if bytes[..2] != [DRAFT_TAG, DRAFT_VERSION] {
            return Err(FrameError::Header);
        }
        Ok(Self {
            expiry_slot: u64::from_le_bytes(*read(bytes, EXPIRY)?),
            tracking_revision: u64::from_le_bytes(*read(bytes, TRACKING_REVISION)?),
            tracking_root: read(bytes, TRACKING_ROOT)?,
            owner_utxo_hash: read(bytes, OWNER_UTXO_HASH)?,
            tx_viewing_pubkey: read(bytes, VIEWING_KEY)?,
            salt: read(bytes, SALT)?,
            recipient_tag: read(bytes, RECIPIENT_TAG)?,
            recipient_zero_ciphertext: read(bytes, ZERO_RECORD)?,
            auditor_ciphertext: read(bytes, AUDITOR_ENVELOPE)?,
            descriptor_proof_bytes: read(bytes, PROOF)?,
        })
    }

    pub fn encode(&self) -> [u8; FRAME_LEN] {
        let mut bytes = [0; FRAME_LEN];
        bytes[..2].copy_from_slice(&[DRAFT_TAG, DRAFT_VERSION]);
        write(&mut bytes, EXPIRY, &self.expiry_slot.to_le_bytes());
        write(&mut bytes, TRACKING_REVISION, &self.tracking_revision.to_le_bytes());
        write(&mut bytes, TRACKING_ROOT, self.tracking_root);
        write(&mut bytes, OWNER_UTXO_HASH, self.owner_utxo_hash);
        write(&mut bytes, VIEWING_KEY, self.tx_viewing_pubkey);
        write(&mut bytes, SALT, self.salt);
        write(&mut bytes, RECIPIENT_TAG, self.recipient_tag);
        write(&mut bytes, ZERO_RECORD, self.recipient_zero_ciphertext);
        write(&mut bytes, AUDITOR_ENVELOPE, self.auditor_ciphertext);
        write(&mut bytes, PROOF, self.descriptor_proof_bytes);
        bytes
    }
}

/// Roles are positions, not addresses or trusted runtime account handles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Role {
    Instructions = 0,
    Depositor = 1,
    InputSource = 2,
    OutputStaging = 3,
    InputMint = 4,
    OutputMint = 5,
    RingPolicy = 6,
    TrackingRegistry = 7,
    FeePayer = 8,
    FeeReceiver = 9,
    Tree = 10,
    RingConfig = 11,
    ShieldedPoolProgram = 12,
    LegacyTokenProgram = 13,
    OutputVault = 14,
    ValueGuardProgram = 15,
    JupiterProgram = 16,
}

/// Effective outer-message roles in the proposed fixed profile. The slot list
/// does not construct accounts, request privileges, or authenticate a message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
    ReadOnly,
    Writable,
    ReadOnlySigner,
    WritableSigner,
}

pub const ROLES: [(Role, Access); 17] = [
    (Role::Instructions, Access::ReadOnly),
    (Role::Depositor, Access::ReadOnlySigner),
    (Role::InputSource, Access::Writable),
    (Role::OutputStaging, Access::Writable),
    (Role::InputMint, Access::ReadOnly),
    (Role::OutputMint, Access::ReadOnly),
    (Role::RingPolicy, Access::ReadOnly),
    (Role::TrackingRegistry, Access::ReadOnly),
    (Role::FeePayer, Access::WritableSigner),
    (Role::FeeReceiver, Access::Writable),
    (Role::Tree, Access::Writable),
    (Role::RingConfig, Access::ReadOnly),
    (Role::ShieldedPoolProgram, Access::ReadOnly),
    (Role::LegacyTokenProgram, Access::ReadOnly),
    (Role::OutputVault, Access::Writable),
    (Role::ValueGuardProgram, Access::ReadOnly),
    (Role::JupiterProgram, Access::ReadOnly),
];

/// Ordered roles used by stock SPP's one-SPL-asset deposit. RingConfig is not
/// an outer signer; only the existing checked RingAuth at SPP slot 2 signs.
pub const SPP_SPL_DEPOSIT_ROLES: [Role; 8] = [
    Role::Tree,
    Role::Depositor,
    Role::RingConfig,
    Role::ShieldedPoolProgram,
    Role::LegacyTokenProgram,
    Role::OutputMint,
    Role::OutputStaging,
    Role::OutputVault,
];
