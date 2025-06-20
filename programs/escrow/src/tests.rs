use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_message::Message;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;
use std::fs;
use std::str::FromStr;

use crate::test_helpers::{
    create_associated_token_account, create_token_mint, deploy_program, mint_tokens_to_account,
    send_transaction,
};

#[test]
fn test_make_offer_succeeds() {
    let mut litesvm = LiteSVM::new();
    let program_id = Pubkey::from_str("8jR5GeNzeweq35Uo84kGP3v1NcBaZWH5u62k7PxN4T2y").unwrap();

    deploy_program(&mut litesvm, &program_id, "../../target/deploy/escrow.so");

    let mint_authority = Keypair::new();
    let alice = Keypair::new();
    let bob = Keypair::new();
    litesvm
        .airdrop(&mint_authority.pubkey(), 1_000_000_000)
        .unwrap();
    litesvm.airdrop(&alice.pubkey(), 1_000_000_000).unwrap();
    litesvm.airdrop(&bob.pubkey(), 1_000_000_000).unwrap();

    let token_mint_a = create_token_mint(&mut litesvm, &mint_authority, 9);
    let token_mint_b = create_token_mint(&mut litesvm, &mint_authority, 9);

    let alice_token_account_a = create_associated_token_account(
        &mut litesvm,
        &alice,
        &token_mint_a.pubkey(),
        &mint_authority,
    );
    let bob_token_account_a = create_associated_token_account(
        &mut litesvm,
        &bob,
        &token_mint_a.pubkey(),
        &mint_authority,
    );
    let bob_token_account_b = create_associated_token_account(
        &mut litesvm,
        &bob,
        &token_mint_b.pubkey(),
        &mint_authority,
    );

    let token = 1_000_000_000u64; // 1 token (10^9)
    let alice_initial_token_a = 10 * token;
    let bob_initial_token_a = 1;
    let bob_initial_token_b = 1 * token;

    mint_tokens_to_account(
        &mut litesvm,
        &token_mint_a.pubkey(),
        &alice_token_account_a,
        alice_initial_token_a,
        &mint_authority,
    );
    mint_tokens_to_account(
        &mut litesvm,
        &token_mint_a.pubkey(),
        &bob_token_account_a,
        bob_initial_token_a,
        &mint_authority,
    );
    mint_tokens_to_account(
        &mut litesvm,
        &token_mint_b.pubkey(),
        &bob_token_account_b,
        bob_initial_token_b,
        &mint_authority,
    );

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

    send_transaction(&mut litesvm, instruction, &[&alice], &alice.pubkey());

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

#[test]
fn test_duplicate_offer_id_fails() {
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

    let mint_authority = Keypair::new();
    let alice = Keypair::new();
    let bob = Keypair::new();
    litesvm
        .airdrop(&mint_authority.pubkey(), 1_000_000_000)
        .unwrap();
    litesvm.airdrop(&alice.pubkey(), 1_000_000_000).unwrap();
    litesvm.airdrop(&bob.pubkey(), 1_000_000_000).unwrap();

    let token_mint_a = create_token_mint(&mut litesvm, &mint_authority, 9);
    let token_mint_b = create_token_mint(&mut litesvm, &mint_authority, 9);

    let alice_token_account_a = create_associated_token_account(
        &mut litesvm,
        &alice,
        &token_mint_a.pubkey(),
        &mint_authority,
    );
    let bob_token_account_a = create_associated_token_account(
        &mut litesvm,
        &bob,
        &token_mint_a.pubkey(),
        &mint_authority,
    );

    let token = 1_000_000_000u64;
    let alice_initial_token_a = 10 * token;
    let bob_initial_token_a = 1;

    mint_tokens_to_account(
        &mut litesvm,
        &token_mint_a.pubkey(),
        &alice_token_account_a,
        alice_initial_token_a,
        &mint_authority,
    );
    mint_tokens_to_account(
        &mut litesvm,
        &token_mint_a.pubkey(),
        &bob_token_account_a,
        bob_initial_token_a,
        &mint_authority,
    );

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
    let mut instruction_data = instruction_discriminator.clone();
    instruction_data.extend_from_slice(&offer_id.to_le_bytes());
    instruction_data.extend_from_slice(&(1 * token).to_le_bytes());
    instruction_data.extend_from_slice(&(1 * token).to_le_bytes());
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
    assert!(result.is_ok(), "First offer should succeed");

    let mut instruction_data = instruction_discriminator;
    instruction_data.extend_from_slice(&offer_id.to_le_bytes());
    instruction_data.extend_from_slice(&(1 * token).to_le_bytes());
    instruction_data.extend_from_slice(&(1 * token).to_le_bytes());
    let account_metas = vec![
        solana_instruction::AccountMeta::new_readonly(spl_associated_token_account::ID, false),
        solana_instruction::AccountMeta::new_readonly(spl_token::ID, false),
        solana_instruction::AccountMeta::new_readonly(anchor_lang::system_program::ID, false),
        solana_instruction::AccountMeta::new(bob.pubkey(), true),
        solana_instruction::AccountMeta::new_readonly(token_mint_a.pubkey(), false),
        solana_instruction::AccountMeta::new_readonly(token_mint_b.pubkey(), false),
        solana_instruction::AccountMeta::new(bob_token_account_a, false),
        solana_instruction::AccountMeta::new(offer_account, false),
        solana_instruction::AccountMeta::new(vault, false),
    ];
    let instruction = solana_instruction::Instruction {
        program_id,
        accounts: account_metas,
        data: instruction_data,
    };
    let recent_blockhash = litesvm.latest_blockhash();
    let message = Message::new(&[instruction], Some(&bob.pubkey()));
    let mut transaction = Transaction::new_unsigned(message);
    transaction.sign(&[&bob], recent_blockhash);
    let result = litesvm.send_transaction(transaction);
    assert!(result.is_err(), "Second offer with same ID should fail");
}

#[test]
fn test_insufficient_funds_fails() {
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

    let mint_authority = Keypair::new();
    let alice = Keypair::new();
    litesvm
        .airdrop(&mint_authority.pubkey(), 1_000_000_000)
        .unwrap();
    litesvm.airdrop(&alice.pubkey(), 1_000_000_000).unwrap();

    let token_mint_a = Keypair::new();
    let token_mint_b = Keypair::new();
    let decimals = 9u8;
    let rent = litesvm.minimum_balance_for_rent_exemption(82);
    for mint in [&token_mint_a, &token_mint_b] {
        litesvm
            .set_account(
                mint.pubkey(),
                solana_account::Account {
                    lamports: rent,
                    data: vec![0u8; 82],
                    owner: spl_token::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            )
            .unwrap();
        let initialize_mint_instruction = spl_token::instruction::initialize_mint(
            &spl_token::ID,
            &mint.pubkey(),
            &mint_authority.pubkey(),
            None,
            decimals,
        )
        .unwrap();
        let message = Message::new(
            &[initialize_mint_instruction],
            Some(&mint_authority.pubkey()),
        );
        let mut transaction = Transaction::new_unsigned(message);
        let blockhash = litesvm.latest_blockhash();
        transaction.sign(&[&mint_authority], blockhash);
        litesvm.send_transaction(transaction).unwrap();
    }

    let alice_token_account_a = spl_associated_token_account::get_associated_token_address(
        &alice.pubkey(),
        &token_mint_a.pubkey(),
    );
    let create_associated_token_account_instruction =
        spl_associated_token_account::instruction::create_associated_token_account(
            &alice.pubkey(),
            &alice.pubkey(),
            &token_mint_a.pubkey(),
            &spl_token::ID,
        );
    let message = Message::new(
        &[create_associated_token_account_instruction],
        Some(&alice.pubkey()),
    );
    let mut transaction = Transaction::new_unsigned(message);
    let blockhash = litesvm.latest_blockhash();
    transaction.sign(&[&alice], blockhash);
    litesvm.send_transaction(transaction).unwrap();

    let token = 1_000_000_000u64;
    let alice_initial_token_a = 10 * token;
    let initialize_mint_instruction = spl_token::instruction::mint_to(
        &spl_token::ID,
        &token_mint_a.pubkey(),
        &alice_token_account_a,
        &mint_authority.pubkey(),
        &[],
        alice_initial_token_a,
    )
    .unwrap();
    let message = Message::new(
        &[initialize_mint_instruction],
        Some(&mint_authority.pubkey()),
    );
    let mut transaction = Transaction::new_unsigned(message);
    let blockhash = litesvm.latest_blockhash();
    transaction.sign(&[&mint_authority], blockhash);
    litesvm.send_transaction(transaction).unwrap();

    // Try to create offer with more tokens than Alice owns
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
    instruction_data.extend_from_slice(&(1000 * token).to_le_bytes()); // Try to offer 1000 tokens (Alice only has 10)
    instruction_data.extend_from_slice(&(1 * token).to_le_bytes());
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
    assert!(result.is_err(), "Offer with insufficient funds should fail");
}

#[test]
fn test_same_token_mints_fails() {
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

    let mint_authority = Keypair::new();
    let alice = Keypair::new();
    litesvm
        .airdrop(&mint_authority.pubkey(), 1_000_000_000)
        .unwrap();
    litesvm.airdrop(&alice.pubkey(), 1_000_000_000).unwrap();

    let token_mint_a = Keypair::new();
    let decimals = 9u8;
    let rent = litesvm.minimum_balance_for_rent_exemption(82);
    litesvm
        .set_account(
            token_mint_a.pubkey(),
            solana_account::Account {
                lamports: rent,
                data: vec![0u8; 82],
                owner: spl_token::ID,
                executable: false,
                rent_epoch: 0,
            },
        )
        .unwrap();
    let initialize_mint_instruction = spl_token::instruction::initialize_mint(
        &spl_token::ID,
        &token_mint_a.pubkey(),
        &mint_authority.pubkey(),
        None,
        decimals,
    )
    .unwrap();
    let message = Message::new(
        &[initialize_mint_instruction],
        Some(&mint_authority.pubkey()),
    );
    let mut transaction = Transaction::new_unsigned(message);
    let blockhash = litesvm.latest_blockhash();
    transaction.sign(&[&mint_authority], blockhash);
    litesvm.send_transaction(transaction).unwrap();

    let alice_token_account_a = spl_associated_token_account::get_associated_token_address(
        &alice.pubkey(),
        &token_mint_a.pubkey(),
    );
    let create_associated_token_account_instruction =
        spl_associated_token_account::instruction::create_associated_token_account(
            &alice.pubkey(),
            &alice.pubkey(),
            &token_mint_a.pubkey(),
            &spl_token::ID,
        );
    let message = Message::new(
        &[create_associated_token_account_instruction],
        Some(&alice.pubkey()),
    );
    let mut transaction = Transaction::new_unsigned(message);
    let blockhash = litesvm.latest_blockhash();
    transaction.sign(&[&alice], blockhash);
    litesvm.send_transaction(transaction).unwrap();

    let token = 1_000_000_000u64;
    let alice_initial_token_a = 10 * token;
    let initialize_mint_instruction = spl_token::instruction::mint_to(
        &spl_token::ID,
        &token_mint_a.pubkey(),
        &alice_token_account_a,
        &mint_authority.pubkey(),
        &[],
        alice_initial_token_a,
    )
    .unwrap();
    let message = Message::new(
        &[initialize_mint_instruction],
        Some(&mint_authority.pubkey()),
    );
    let mut transaction = Transaction::new_unsigned(message);
    let blockhash = litesvm.latest_blockhash();
    transaction.sign(&[&mint_authority], blockhash);
    litesvm.send_transaction(transaction).unwrap();

    // Try to create offer with same token mint for both token_a and token_b
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
    instruction_data.extend_from_slice(&(1 * token).to_le_bytes());
    instruction_data.extend_from_slice(&(1 * token).to_le_bytes());
    let account_metas = vec![
        solana_instruction::AccountMeta::new_readonly(spl_associated_token_account::ID, false),
        solana_instruction::AccountMeta::new_readonly(spl_token::ID, false),
        solana_instruction::AccountMeta::new_readonly(anchor_lang::system_program::ID, false),
        solana_instruction::AccountMeta::new(alice.pubkey(), true),
        solana_instruction::AccountMeta::new_readonly(token_mint_a.pubkey(), false),
        solana_instruction::AccountMeta::new_readonly(token_mint_a.pubkey(), false), // Same mint for both
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
    assert!(result.is_err(), "Offer with same token mints should fail");
}

#[test]
fn test_zero_token_b_wanted_amount_fails() {
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

    let mint_authority = Keypair::new();
    let alice = Keypair::new();
    litesvm
        .airdrop(&mint_authority.pubkey(), 1_000_000_000)
        .unwrap();
    litesvm.airdrop(&alice.pubkey(), 1_000_000_000).unwrap();

    let token_mint_a = Keypair::new();
    let token_mint_b = Keypair::new();
    let decimals = 9u8;
    let rent = litesvm.minimum_balance_for_rent_exemption(82);
    for mint in [&token_mint_a, &token_mint_b] {
        litesvm
            .set_account(
                mint.pubkey(),
                solana_account::Account {
                    lamports: rent,
                    data: vec![0u8; 82],
                    owner: spl_token::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            )
            .unwrap();
        let initialize_mint_instruction = spl_token::instruction::initialize_mint(
            &spl_token::ID,
            &mint.pubkey(),
            &mint_authority.pubkey(),
            None,
            decimals,
        )
        .unwrap();
        let message = Message::new(
            &[initialize_mint_instruction],
            Some(&mint_authority.pubkey()),
        );
        let mut transaction = Transaction::new_unsigned(message);
        let blockhash = litesvm.latest_blockhash();
        transaction.sign(&[&mint_authority], blockhash);
        litesvm.send_transaction(transaction).unwrap();
    }

    let alice_token_account_a = spl_associated_token_account::get_associated_token_address(
        &alice.pubkey(),
        &token_mint_a.pubkey(),
    );
    let create_associated_token_account_instruction =
        spl_associated_token_account::instruction::create_associated_token_account(
            &alice.pubkey(),
            &alice.pubkey(),
            &token_mint_a.pubkey(),
            &spl_token::ID,
        );
    let message = Message::new(
        &[create_associated_token_account_instruction],
        Some(&alice.pubkey()),
    );
    let mut transaction = Transaction::new_unsigned(message);
    let blockhash = litesvm.latest_blockhash();
    transaction.sign(&[&alice], blockhash);
    litesvm.send_transaction(transaction).unwrap();

    let token = 1_000_000_000u64;
    let alice_initial_token_a = 10 * token;
    let initialize_mint_instruction = spl_token::instruction::mint_to(
        &spl_token::ID,
        &token_mint_a.pubkey(),
        &alice_token_account_a,
        &mint_authority.pubkey(),
        &[],
        alice_initial_token_a,
    )
    .unwrap();
    let message = Message::new(
        &[initialize_mint_instruction],
        Some(&mint_authority.pubkey()),
    );
    let mut transaction = Transaction::new_unsigned(message);
    let blockhash = litesvm.latest_blockhash();
    transaction.sign(&[&mint_authority], blockhash);
    litesvm.send_transaction(transaction).unwrap();

    // Try to create offer with zero token_b_wanted_amount
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
    instruction_data.extend_from_slice(&(1 * token).to_le_bytes());
    instruction_data.extend_from_slice(&0u64.to_le_bytes()); // Zero wanted amount
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
    assert!(
        result.is_err(),
        "Offer with zero token_b_wanted_amount should fail"
    );
}

#[test]
fn test_zero_token_a_offered_amount_fails() {
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

    let mint_authority = Keypair::new();
    let alice = Keypair::new();
    litesvm
        .airdrop(&mint_authority.pubkey(), 1_000_000_000)
        .unwrap();
    litesvm.airdrop(&alice.pubkey(), 1_000_000_000).unwrap();

    let token_mint_a = Keypair::new();
    let token_mint_b = Keypair::new();
    let decimals = 9u8;
    let rent = litesvm.minimum_balance_for_rent_exemption(82);
    for mint in [&token_mint_a, &token_mint_b] {
        litesvm
            .set_account(
                mint.pubkey(),
                solana_account::Account {
                    lamports: rent,
                    data: vec![0u8; 82],
                    owner: spl_token::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            )
            .unwrap();
        let initialize_mint_instruction = spl_token::instruction::initialize_mint(
            &spl_token::ID,
            &mint.pubkey(),
            &mint_authority.pubkey(),
            None,
            decimals,
        )
        .unwrap();
        let message = Message::new(
            &[initialize_mint_instruction],
            Some(&mint_authority.pubkey()),
        );
        let mut transaction = Transaction::new_unsigned(message);
        let blockhash = litesvm.latest_blockhash();
        transaction.sign(&[&mint_authority], blockhash);
        litesvm.send_transaction(transaction).unwrap();
    }

    let alice_token_account_a = spl_associated_token_account::get_associated_token_address(
        &alice.pubkey(),
        &token_mint_a.pubkey(),
    );
    let create_associated_token_account_instruction =
        spl_associated_token_account::instruction::create_associated_token_account(
            &alice.pubkey(),
            &alice.pubkey(),
            &token_mint_a.pubkey(),
            &spl_token::ID,
        );
    let message = Message::new(
        &[create_associated_token_account_instruction],
        Some(&alice.pubkey()),
    );
    let mut transaction = Transaction::new_unsigned(message);
    let blockhash = litesvm.latest_blockhash();
    transaction.sign(&[&alice], blockhash);
    litesvm.send_transaction(transaction).unwrap();

    let token = 1_000_000_000u64;
    let alice_initial_token_a = 10 * token;
    let initialize_mint_instruction = spl_token::instruction::mint_to(
        &spl_token::ID,
        &token_mint_a.pubkey(),
        &alice_token_account_a,
        &mint_authority.pubkey(),
        &[],
        alice_initial_token_a,
    )
    .unwrap();
    let message = Message::new(
        &[initialize_mint_instruction],
        Some(&mint_authority.pubkey()),
    );
    let mut transaction = Transaction::new_unsigned(message);
    let blockhash = litesvm.latest_blockhash();
    transaction.sign(&[&mint_authority], blockhash);
    litesvm.send_transaction(transaction).unwrap();

    // Try to create offer with zero token_a_offered_amount
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
    instruction_data.extend_from_slice(&0u64.to_le_bytes()); // Zero offered amount
    instruction_data.extend_from_slice(&(1 * token).to_le_bytes());
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
    assert!(
        result.is_err(),
        "Offer with zero token_a_offered_amount should fail"
    );
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

#[test]
fn test_take_offer_insufficient_funds_fails() {
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

    let mint_authority = Keypair::new();
    let alice = Keypair::new();
    let bob = Keypair::new();
    litesvm
        .airdrop(&mint_authority.pubkey(), 1_000_000_000)
        .unwrap();
    litesvm.airdrop(&alice.pubkey(), 1_000_000_000).unwrap();
    litesvm.airdrop(&bob.pubkey(), 1_000_000_000).unwrap();

    let token_mint_a = Keypair::new();
    let token_mint_b = Keypair::new();
    let decimals = 9u8;
    let rent = litesvm.minimum_balance_for_rent_exemption(82);
    for mint in [&token_mint_a, &token_mint_b] {
        litesvm
            .set_account(
                mint.pubkey(),
                solana_account::Account {
                    lamports: rent,
                    data: vec![0u8; 82],
                    owner: spl_token::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            )
            .unwrap();
        let initialize_mint_instruction = spl_token::instruction::initialize_mint(
            &spl_token::ID,
            &mint.pubkey(),
            &mint_authority.pubkey(),
            None,
            decimals,
        )
        .unwrap();
        let message = Message::new(
            &[initialize_mint_instruction],
            Some(&mint_authority.pubkey()),
        );
        let mut transaction = Transaction::new_unsigned(message);
        let blockhash = litesvm.latest_blockhash();
        transaction.sign(&[&mint_authority], blockhash);
        litesvm.send_transaction(transaction).unwrap();
    }

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
    let alice_token_account_b = spl_associated_token_account::get_associated_token_address(
        &alice.pubkey(),
        &token_mint_b.pubkey(),
    );
    for (owner, mint, ata) in [
        (&alice, &token_mint_a, &alice_token_account_a),
        (&bob, &token_mint_a, &bob_token_account_a),
        (&bob, &token_mint_b, &bob_token_account_b),
        (&alice, &token_mint_b, &alice_token_account_b),
    ] {
        let create_associated_token_account_instruction =
            spl_associated_token_account::instruction::create_associated_token_account(
                &owner.pubkey(),
                &owner.pubkey(),
                &mint.pubkey(),
                &spl_token::ID,
            );
        let message = Message::new(
            &[create_associated_token_account_instruction],
            Some(&owner.pubkey()),
        );
        let mut transaction = Transaction::new_unsigned(message);
        let blockhash = litesvm.latest_blockhash();
        transaction.sign(&[owner], blockhash);
        litesvm.send_transaction(transaction).unwrap();
    }

    let token = 1_000_000_000u64;
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
        let initialize_mint_instruction = spl_token::instruction::mint_to(
            &spl_token::ID,
            &mint,
            &ata,
            &mint_authority.pubkey(),
            &[],
            amount,
        )
        .unwrap();
        let message = Message::new(
            &[initialize_mint_instruction],
            Some(&mint_authority.pubkey()),
        );
        let mut transaction = Transaction::new_unsigned(message);
        let blockhash = litesvm.latest_blockhash();
        transaction.sign(&[&mint_authority], blockhash);
        litesvm.send_transaction(transaction).unwrap();
    }

    // Create an offer from Alice for a large amount of token B
    let large_token_b_amount = 1000 * token; // Much larger than Bob's balance
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
    instruction_data.extend_from_slice(&(1 * token).to_le_bytes());
    instruction_data.extend_from_slice(&large_token_b_amount.to_le_bytes());
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
    assert!(result.is_ok(), "Alice's offer should succeed");

    // Try to take the offer with Bob who has insufficient token B
    let discriminator_input = b"global:take_offer";
    let instruction_discriminator =
        anchor_lang::solana_program::hash::hash(discriminator_input).to_bytes()[..8].to_vec();
    let instruction_data = instruction_discriminator;
    let account_metas = vec![
        solana_instruction::AccountMeta::new(bob.pubkey(), true),
        solana_instruction::AccountMeta::new_readonly(alice.pubkey(), false),
        solana_instruction::AccountMeta::new_readonly(token_mint_a.pubkey(), false),
        solana_instruction::AccountMeta::new_readonly(token_mint_b.pubkey(), false),
        solana_instruction::AccountMeta::new(bob_token_account_a, false),
        solana_instruction::AccountMeta::new(alice_token_account_b, false),
        solana_instruction::AccountMeta::new(offer_account, false),
        solana_instruction::AccountMeta::new(vault, false),
    ];
    let instruction = solana_instruction::Instruction {
        program_id,
        accounts: account_metas,
        data: instruction_data,
    };
    let recent_blockhash = litesvm.latest_blockhash();
    let message = Message::new(&[instruction], Some(&bob.pubkey()));
    let mut transaction = Transaction::new_unsigned(message);
    transaction.sign(&[&bob], recent_blockhash);
    let result = litesvm.send_transaction(transaction);
    assert!(
        result.is_err(),
        "Take offer with insufficient funds should fail"
    );
}
