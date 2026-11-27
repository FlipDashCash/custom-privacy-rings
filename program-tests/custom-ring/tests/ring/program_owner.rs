//! A plain program's PDA receives a ring note and spends it through CPI, with real proofs.

use solana_instruction::AccountMeta;

use super::*;

/// `examples/enrollment-vault`, its manager moves the notes of its `owner` PDA.
const VAULT_PROGRAM: Address = Address::new_from_array([0x63; 32]);
const DEPOSITED: u64 = 1_000_000;
const PAID: u64 = 600_000;
const SPENT: u64 = 250_000;

/// `transfer` forwarded by the vault, its PDA signs the inner call.
fn through_vault(mut transfer: Instruction, manager: Address, vault: Address) -> Instruction {
    let ring_program = transfer.program_id;
    for meta in &mut transfer.accounts {
        if meta.pubkey == vault {
            meta.is_signer = false;
        }
    }
    let mut accounts = vec![
        AccountMeta::new_readonly(manager, true),
        AccountMeta::new_readonly(vault, false),
        AccountMeta::new_readonly(ring_program, false),
    ];
    accounts.append(&mut transfer.accounts);
    Instruction {
        program_id: VAULT_PROGRAM,
        accounts,
        data: transfer.data,
    }
}

#[test]
fn a_program_pda_receives_and_spends_a_ring_note_through_cpi() -> Result<()> {
    let env = setup_with_programs(&[(VAULT_PROGRAM, "target/deploy/enrollment_vault.so")])?;
    let rpc = env.client.rpc();
    let ring_program = custom_ring_program_id()?;
    let ring = CustomRing::new(ring_program);
    let auditor = X25519Keypair::new();
    RegisterRing {
        ring,
        payer: &env.payer,
        auditor_pubkey: auditor.pubkey(),
        tier: Tier::policy(&EMPTY),
    }
    .send(&env.client)?;
    enroll_actors(&env, ring)?;
    let sender = &env.sender.keypair;
    let manager = env.payer.pubkey();
    let vault = Address::find_program_address(&[b"owner"], &VAULT_PROGRAM).0;
    let state = [manager, ring_program, ring.tracking_registry()?].map(|key| key.to_bytes());
    env.create_account(vault, VAULT_PROGRAM, state.as_flattened())?;
    let holder = pda_keys(&vault, &env.recipient.keypair)?;
    let holder_address = holder.address()?;
    let tree_id = env.tree_id(env.tree)?;

    // 1. A wallet pays the PDA inside the ring.
    let deposited = RingNotes {
        ring,
        owner: sender,
        amount: DEPOSITED,
        asset: DepositAsset::Sol,
        count: 1,
        env: &env,
    }
    .deposit()?;
    let input = indexed_note(
        deposited.into_iter().next().context("deposited input")?,
        sender,
        &env,
        env.tree,
    )?;
    let mut payment =
        ConfidentialTransaction::new_with_ring(vec![input], sender.pubkey(), ring_program)?
            .with_output_tree_id(tree_id)?;
    payment.transfer_sol(&holder_address, PAID)?;
    let receipt = OwnerTransfer {
        ring,
        sender,
        prepared: payment,
        interface_transfer_accounts: Vec::new(),
        auditor_tag: auditor_view_tag(&auditor.pubkey()),
        cosigner: None,
    }
    .send(&env)?;
    let paid = receipt
        .outputs
        .iter()
        .find(|output| output.owner_address == Some(holder_address))
        .context("the output the PDA owns")?;
    let note = pda_output(&env, paid, &holder_address)?;

    // 2. The manager proves a spend of the PDA's note, the PDA is the one owner signer.
    let mut spend = ConfidentialTransaction::new_with_ring(vec![note], manager, ring_program)?
        .with_output_tree_id(tree_id)?;
    spend.transfer_sol(&sender.shielded_address()?, SPENT)?;
    let zero_key = NullifierKey::from_secret([0; 31]);
    let transfer = CustomRingTransfer::new(CustomRingTransferInput {
        ring,
        sender: &holder,
        nullifier_key: Some(&zero_key),
        transaction: spend,
    })
    .spend_public(vault);
    let proven = env.run(transfer.prove(env.client.ring_client()))?;
    ensure!(proven.owner_signers() == [vault], "the PDA alone signs");
    let direct = proven.instruction()?;

    // 3. No key signs for the PDA, the top-level send is refused.
    let mut unsigned = direct.clone();
    for meta in &mut unsigned.accounts {
        if meta.pubkey == vault {
            meta.is_signer = false;
        }
    }
    send_expecting_rejection(rpc, &env.payer, unsigned)?;

    // 4. The vault forwards the same instruction and its PDA signs through CPI.
    let signature = send(rpc, &env.payer, &[through_vault(direct, manager, vault)])?;
    assert_transaction_compute_units(
        rpc,
        &signature,
        "PDA owner transfer, one CPI hop",
        u64::from(compute_unit_limit(InstructionTag::Transfer)),
    )?;

    // 5. The auditor opens the CPI-nested transfer, the payment and the PDA's zero-key change.
    let audited = AuditLookup {
        scan: RingScan::new(ring_program, auditor.pubkey()).with_mode(RingMode::Confidential),
        auditor: &auditor,
        signature,
    }
    .run(&env)?;
    let opened: Vec<(u64, Option<[u8; 32]>)> = audited
        .outputs
        .iter()
        .filter_map(|output| output.opening.utxo())
        .map(|output| (output.amount, output.owner.nullifier_pk()))
        .collect();
    ensure!(opened.iter().any(|(amount, _)| *amount == SPENT));
    ensure!(opened
        .iter()
        .any(|(amount, key)| *amount == PAID - SPENT && *key == Some(ZERO_NULLIFIER_PUBKEY)));
    Ok(())
}
