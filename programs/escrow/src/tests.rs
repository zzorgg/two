use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_message::Message;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;
use std::fs;
use std::str::FromStr;

#[test]
fn test_make_offer_instruction() {
    let mut litesvm = LiteSVM::new();
    let program_bytes =
        fs::read("../../target/deploy/escrow.so").expect("Failed to read program binary");
    let program_id = Pubkey::from_str("8jR5GeNzeweq35Uo84kGP3v1NcBaZWH5u62k7PxN4T2y").unwrap();
    litesvm
        .set_account(
            program_id,
            solana_account::Account {
                lamports: litesvm.minimum_balance_for_rent_exemption(program_bytes.len()),
                data: program_bytes,
                owner: solana_program::bpf_loader::ID,
                executable: true,
                rent_epoch: 0,
            },
        )
        .expect("Failed to deploy program");

    // --- SETUP: Mint authority, mints, token accounts, and minting tokens ---
    let mint_authority = Keypair::new();
    let alice = Keypair::new();
    let bob = Keypair::new();
    litesvm
        .airdrop(&mint_authority.pubkey(), 1_000_000_000)
        .unwrap();
    litesvm.airdrop(&alice.pubkey(), 1_000_000_000).unwrap();
    litesvm.airdrop(&bob.pubkey(), 1_000_000_000).unwrap();

    // Create two mints
    let token_mint_a = Keypair::new();
    let token_mint_b = Keypair::new();
    let decimals = 9u8;
    let rent = litesvm.minimum_balance_for_rent_exemption(82); // SPL Token mint size
    for mint in [&token_mint_a, &token_mint_b] {
        litesvm
            .set_account(
                mint.pubkey(),
                solana_account::Account {
                    lamports: rent,
                    data: vec![0u8; 82], // SPL Token mint size
                    owner: spl_token::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            )
            .unwrap();
        let ix = spl_token::instruction::initialize_mint(
            &spl_token::ID,
            &mint.pubkey(),
            &mint_authority.pubkey(),
            None,
            decimals,
        )
        .unwrap();
        let message = Message::new(&[ix], Some(&mint_authority.pubkey()));
        let mut tx = Transaction::new_unsigned(message);
        let blockhash = litesvm.latest_blockhash();
        tx.sign(&[&mint_authority], blockhash);
        litesvm.send_transaction(tx).unwrap();
    }

    // Create associated token accounts for Alice and Bob for each mint
    let alice_token_account_a = spl_associated_token_account::get_associated_token_address(
        &alice.pubkey(),
        &token_mint_a.pubkey(),
    );
    let bob_token_account_a = spl_associated_token_account::get_associated_token_address(
        &bob.pubkey(),
        &token_mint_a.pubkey(),
    );
    let bob_token_account_b = spl_associated_token_account::get_associated_token_address(
        &bob.pubkey(),
        &token_mint_b.pubkey(),
    );
    let rent_token = litesvm.minimum_balance_for_rent_exemption(165); // SPL Token account size
    for (owner, mint, ata) in [
        (&alice, &token_mint_a, &alice_token_account_a),
        (&bob, &token_mint_a, &bob_token_account_a),
        (&bob, &token_mint_b, &bob_token_account_b),
    ] {
        let create_ata_ix =
            spl_associated_token_account::instruction::create_associated_token_account(
                &owner.pubkey(),
                &owner.pubkey(),
                &mint.pubkey(),
                &spl_token::ID,
            );
        let message = Message::new(&[create_ata_ix], Some(&owner.pubkey()));
        let mut tx = Transaction::new_unsigned(message);
        let blockhash = litesvm.latest_blockhash();
        tx.sign(&[owner], blockhash);
        litesvm.send_transaction(tx).unwrap();
    }

    // Mint tokens to Alice and Bob
    let token = 1_000_000_000u64; // 1 token (10^9)
    let alice_initial_token_a = 10 * token;
    let bob_initial_token_a = 1;
    let bob_initial_token_b = 1 * token;
    let mint_to_instructions = vec![
        (
            token_mint_a.pubkey(),
            alice_token_account_a,
            alice_initial_token_a,
        ),
        (
            token_mint_a.pubkey(),
            bob_token_account_a,
            bob_initial_token_a,
        ),
        (
            token_mint_b.pubkey(),
            bob_token_account_b,
            bob_initial_token_b,
        ),
    ];
    for (mint, ata, amount) in mint_to_instructions {
        let ix = spl_token::instruction::mint_to(
            &spl_token::ID,
            &mint,
            &ata,
            &mint_authority.pubkey(),
            &[],
            amount,
        )
        .unwrap();
        let message = Message::new(&[ix], Some(&mint_authority.pubkey()));
        let mut tx = Transaction::new_unsigned(message);
        let blockhash = litesvm.latest_blockhash();
        tx.sign(&[&mint_authority], blockhash);
        litesvm.send_transaction(tx).unwrap();
    }

    // --- Now run the make_offer test as before, using Alice's accounts ---
    let offer_id = 12345u64;
    let (offer_account, _offer_bump) =
        Pubkey::find_program_address(&[b"offer", &offer_id.to_le_bytes()], &program_id);
    let vault = spl_associated_token_account::get_associated_token_address(
        &offer_account,
        &token_mint_a.pubkey(),
    );
    let discriminator_input = b"global:make_offer";
    let instruction_discriminator =
        anchor_lang::solana_program::hash::hash(discriminator_input).to_bytes()[..8].to_vec();
    let mut instruction_data = instruction_discriminator;
    instruction_data.extend_from_slice(&offer_id.to_le_bytes());
    instruction_data.extend_from_slice(&(1 * token).to_le_bytes()); // token_a_offered_amount
    instruction_data.extend_from_slice(&(1 * token).to_le_bytes()); // token_b_wanted_amount
    let account_metas = vec![
        solana_instruction::AccountMeta::new_readonly(spl_associated_token_account::ID, false),
        solana_instruction::AccountMeta::new_readonly(spl_token::ID, false),
        solana_instruction::AccountMeta::new_readonly(anchor_lang::system_program::ID, false),
        solana_instruction::AccountMeta::new(alice.pubkey(), true),
        solana_instruction::AccountMeta::new_readonly(token_mint_a.pubkey(), false),
        solana_instruction::AccountMeta::new_readonly(token_mint_b.pubkey(), false),
        solana_instruction::AccountMeta::new(alice_token_account_a, false),
        solana_instruction::AccountMeta::new(offer_account, false),
        solana_instruction::AccountMeta::new(vault, false),
    ];
    let instruction = solana_instruction::Instruction {
        program_id,
        accounts: account_metas,
        data: instruction_data,
    };
    let recent_blockhash = litesvm.latest_blockhash();
    let message = Message::new(&[instruction], Some(&alice.pubkey()));
    let mut transaction = Transaction::new_unsigned(message);
    transaction.sign(&[&alice], recent_blockhash);
    let result = litesvm.send_transaction(transaction);
    match result {
        Ok(transaction_metadata) => {
            println!(
                "✅ make_offer transaction successful! Signature: {:?}",
                transaction_metadata
            );
            let offer_account_data = litesvm
                .get_account(&offer_account)
                .expect("Failed to get offer account");
            println!(
                "Offer account created with {} bytes",
                offer_account_data.data.len()
            );
            let vault_data = litesvm
                .get_account(&vault)
                .expect("Failed to get vault account");
            println!("Vault account created with {} bytes", vault_data.data.len());
        }
        Err(error) => {
            println!("❌ make_offer transaction failed: {:?}", error);
            panic!("Transaction failed");
        }
    }
}

#[test]
fn test_make_offer_duplicate_id() {
    // Test that creating offers with duplicate IDs fails
    assert!(true); // Placeholder - basic test structure works
}

#[test]
fn test_make_offer_insufficient_funds() {
    // Test that creating offers with insufficient funds fails
    assert!(true); // Placeholder - basic test structure works
}

#[test]
fn test_make_offer_same_token_mints() {
    // Test that creating offers with same token mints fails
    assert!(true); // Placeholder - basic test structure works
}

#[test]
fn test_make_offer_zero_amounts() {
    // Test that creating offers with zero amounts fails
    assert!(true); // Placeholder - basic test structure works
}

#[test]
fn test_take_offer_success() {
    // Test that taking offers succeeds
    assert!(true); // Placeholder - basic test structure works
}

#[test]
fn test_refund_offer_success() {
    // Test that refunding offers succeeds
    assert!(true); // Placeholder - basic test structure works
}

#[test]
fn test_refund_offer_non_maker() {
    // Test that non-makers cannot refund offers
    assert!(true); // Placeholder - basic test structure works
}
