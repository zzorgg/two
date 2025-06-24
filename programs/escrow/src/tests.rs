use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_message::Message;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;
use std::fs;
use std::str::FromStr;

use crate::escrow_test_helpers::{
    build_make_offer_instruction, build_refund_offer_instruction, build_take_offer_instruction,
    setup_escrow_test, MakeOfferAccounts, RefundOfferAccounts, TakeOfferAccounts, TOKEN_A, TOKEN_B,
};
use crate::test_helpers::{
    assert_token_balance, create_associated_token_account, create_token_mint, create_wallet,
    create_wallets, deploy_program, mint_tokens_to_account, send_transaction_from_instructions,
};

#[test]
fn test_make_offer_succeeds() {
    let mut litesvm = LiteSVM::new();
    let program_id = Pubkey::from_str("8jR5GeNzeweq35Uo84kGP3v1NcBaZWH5u62k7PxN4T2y").unwrap();

    deploy_program(&mut litesvm, &program_id, "../../target/deploy/escrow.so");

    let mint_authority = create_wallet(&mut litesvm, 1_000_000_000);
    let [alice, bob] = create_wallets(&mut litesvm, 2, 1_000_000_000)
        .try_into()
        .unwrap();

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

    let alice_initial_token_a = 10 * TOKEN_A;
    let bob_initial_token_a = 1;
    let bob_initial_token_b = 1 * TOKEN_B;

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
    let mut instruction_data = instruction_discriminator.clone();
    instruction_data.extend_from_slice(&offer_id.to_le_bytes());
    instruction_data.extend_from_slice(&(1 * TOKEN_A).to_le_bytes()); // token_a_offered_amount
    instruction_data.extend_from_slice(&(1 * TOKEN_B).to_le_bytes()); // token_b_wanted_amount
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

    send_transaction_from_instructions(&mut litesvm, vec![instruction], &[&alice], &alice.pubkey())
        .unwrap();

    let mut instruction_data = instruction_discriminator.clone();
    instruction_data.extend_from_slice(&offer_id.to_le_bytes());
    instruction_data.extend_from_slice(&(1 * TOKEN_A).to_le_bytes());
    instruction_data.extend_from_slice(&(1 * TOKEN_B).to_le_bytes());
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
    let result =
        send_transaction_from_instructions(&mut litesvm, vec![instruction], &[&bob], &bob.pubkey());
    assert!(result.is_err(), "Second offer with same ID should fail");
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

    let mint_authority = create_wallet(&mut litesvm, 1_000_000_000);
    let [alice, bob] = create_wallets(&mut litesvm, 2, 1_000_000_000)
        .try_into()
        .unwrap();

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

    let alice_initial_token_a = 10 * TOKEN_A;
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
    instruction_data.extend_from_slice(&(1 * TOKEN_A).to_le_bytes());
    instruction_data.extend_from_slice(&(1 * TOKEN_B).to_le_bytes());
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
    let result = send_transaction_from_instructions(
        &mut litesvm,
        vec![instruction],
        &[&alice],
        &alice.pubkey(),
    );
    assert!(result.is_ok(), "First offer should succeed");

    let mut instruction_data = instruction_discriminator;
    instruction_data.extend_from_slice(&offer_id.to_le_bytes());
    instruction_data.extend_from_slice(&(1 * TOKEN_A).to_le_bytes());
    instruction_data.extend_from_slice(&(1 * TOKEN_B).to_le_bytes());
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
    let result =
        send_transaction_from_instructions(&mut litesvm, vec![instruction], &[&bob], &bob.pubkey());
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

    let mint_authority = create_wallet(&mut litesvm, 1_000_000_000);
    let alice = create_wallet(&mut litesvm, 1_000_000_000);

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

    let alice_initial_token_a = 10 * TOKEN_A;
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
    instruction_data.extend_from_slice(&(1000 * TOKEN_A).to_le_bytes()); // Try to offer 1000 tokens (Alice only has 10)
    instruction_data.extend_from_slice(&(1 * TOKEN_B).to_le_bytes());
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
    let result = send_transaction_from_instructions(
        &mut litesvm,
        vec![instruction],
        &[&alice],
        &alice.pubkey(),
    );
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

    let mint_authority = create_wallet(&mut litesvm, 1_000_000_000);
    let alice = create_wallet(&mut litesvm, 1_000_000_000);

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

    let alice_initial_token_a = 10 * TOKEN_A;
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
    instruction_data.extend_from_slice(&(1 * TOKEN_A).to_le_bytes());
    instruction_data.extend_from_slice(&(1 * TOKEN_B).to_le_bytes());
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
    let result = send_transaction_from_instructions(
        &mut litesvm,
        vec![instruction],
        &[&alice],
        &alice.pubkey(),
    );
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

    let mint_authority = create_wallet(&mut litesvm, 1_000_000_000);
    let alice = create_wallet(&mut litesvm, 1_000_000_000);

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

    let alice_initial_token_a = 10 * TOKEN_A;
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
    instruction_data.extend_from_slice(&(1 * TOKEN_A).to_le_bytes());
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
    let result = send_transaction_from_instructions(
        &mut litesvm,
        vec![instruction],
        &[&alice],
        &alice.pubkey(),
    );
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

    let mint_authority = create_wallet(&mut litesvm, 1_000_000_000);
    let alice = create_wallet(&mut litesvm, 1_000_000_000);

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

    let alice_initial_token_a = 10 * TOKEN_A;
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
    instruction_data.extend_from_slice(&(1 * TOKEN_B).to_le_bytes());
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
    let result = send_transaction_from_instructions(
        &mut litesvm,
        vec![instruction],
        &[&alice],
        &alice.pubkey(),
    );
    assert!(
        result.is_err(),
        "Offer with zero token_a_offered_amount should fail"
    );
}

#[test]
fn test_take_offer_success() {
    let mut test_environment = setup_escrow_test();

    // Alice creates an offer: 3 token A for 2 token B
    let offer_id = 55555u64;
    let (offer_account, _offer_bump) = Pubkey::find_program_address(
        &[b"offer", &offer_id.to_le_bytes()],
        &test_environment.program_id,
    );
    let vault = spl_associated_token_account::get_associated_token_address(
        &offer_account,
        &test_environment.token_mint_a.pubkey(),
    );

    let make_offer_accounts = MakeOfferAccounts {
        associated_token_program: spl_associated_token_account::ID,
        token_program: spl_token::ID,
        system_program: anchor_lang::system_program::ID,
        maker: test_environment.alice.pubkey(),
        token_mint_a: test_environment.token_mint_a.pubkey(),
        token_mint_b: test_environment.token_mint_b.pubkey(),
        maker_token_account_a: test_environment.alice_token_account_a,
        offer_account,
        vault,
    };

    let make_offer_instruction = build_make_offer_instruction(
        offer_id,
        3 * TOKEN_A, // token_a_offered_amount
        2 * TOKEN_B, // token_b_wanted_amount
        make_offer_accounts,
    );

    send_transaction_from_instructions(
        &mut test_environment.litesvm,
        vec![make_offer_instruction],
        &[&test_environment.alice],
        &test_environment.alice.pubkey(),
    )
    .unwrap();

    // Bob takes the offer
    let take_offer_accounts = TakeOfferAccounts {
        associated_token_program: spl_associated_token_account::ID,
        token_program: spl_token::ID,
        system_program: anchor_lang::system_program::ID,
        taker: test_environment.bob.pubkey(),
        maker: test_environment.alice.pubkey(),
        token_mint_a: test_environment.token_mint_a.pubkey(),
        token_mint_b: test_environment.token_mint_b.pubkey(),
        taker_token_account_a: test_environment.bob_token_account_a,
        taker_token_account_b: test_environment.bob_token_account_b,
        maker_token_account_b: test_environment.alice_token_account_b,
        offer_account,
        vault,
    };

    let take_offer_instruction = build_take_offer_instruction(take_offer_accounts);
    send_transaction_from_instructions(
        &mut test_environment.litesvm,
        vec![take_offer_instruction],
        &[&test_environment.bob],
        &test_environment.bob.pubkey(),
    )
    .unwrap();

    // Check balances
    assert_token_balance(
        &test_environment.litesvm,
        &test_environment.alice_token_account_a,
        7 * TOKEN_A,
        "Alice should have 7 token A left",
    );
    assert_token_balance(
        &test_environment.litesvm,
        &test_environment.alice_token_account_b,
        2 * TOKEN_B,
        "Alice should have received 2 token B",
    );
    assert_token_balance(
        &test_environment.litesvm,
        &test_environment.bob_token_account_a,
        3 * TOKEN_A,
        "Bob should have received 3 token A",
    );
    assert_token_balance(
        &test_environment.litesvm,
        &test_environment.bob_token_account_b,
        3 * TOKEN_B,
        "Bob should have 3 token B left",
    );

    // Optionally, check that the offer account is closed or marked as completed
    let offer_account_data = test_environment.litesvm.get_account(&offer_account);
    assert!(
        offer_account_data.is_none() || offer_account_data.unwrap().data.is_empty(),
        "Offer account should be closed or empty after being taken"
    );
}

#[test]
fn test_refund_offer_success() {
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

    let mint_authority = create_wallet(&mut litesvm, 1_000_000_000);
    let alice = create_wallet(&mut litesvm, 1_000_000_000);
    let bob = create_wallet(&mut litesvm, 1_000_000_000);

    // Create mints
    let token_mint_a = create_token_mint(&mut litesvm, &mint_authority, 9);
    let token_mint_b = create_token_mint(&mut litesvm, &mint_authority, 9);

    // Create token accounts
    let alice_token_account_a = create_associated_token_account(
        &mut litesvm,
        &alice,
        &token_mint_a.pubkey(),
        &mint_authority,
    );

    // Mint tokens to Alice
    let alice_initial_token_a = 10 * TOKEN_A;
    mint_tokens_to_account(
        &mut litesvm,
        &token_mint_a.pubkey(),
        &alice_token_account_a,
        alice_initial_token_a,
        &mint_authority,
    );

    // Alice creates an offer: 3 token A for 2 token B
    let offer_id = 77777u64;
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
    instruction_data.extend_from_slice(&(3 * TOKEN_A).to_le_bytes()); // token_a_offered_amount
    instruction_data.extend_from_slice(&(2 * TOKEN_B).to_le_bytes()); // token_b_wanted_amount
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
    send_transaction_from_instructions(&mut litesvm, vec![instruction], &[&alice], &alice.pubkey())
        .unwrap();

    // Check that Alice's balance decreased after creating the offer
    assert_token_balance(
        &litesvm,
        &alice_token_account_a,
        7 * TOKEN_A,
        "Alice should have 7 token A left after creating offer",
    );

    // Alice refunds the offer
    let refund_offer_accounts = RefundOfferAccounts {
        token_program: spl_token::ID,
        system_program: anchor_lang::system_program::ID,
        maker: alice.pubkey(),
        token_mint_a: token_mint_a.pubkey(),
        maker_token_account_a: alice_token_account_a,
        offer_account,
        vault,
    };

    let refund_instruction = build_refund_offer_instruction(refund_offer_accounts);
    send_transaction_from_instructions(
        &mut litesvm,
        vec![refund_instruction],
        &[&alice],
        &alice.pubkey(),
    )
    .unwrap();

    // Check that Alice's balance is restored after refunding
    assert_token_balance(
        &litesvm,
        &alice_token_account_a,
        10 * TOKEN_A,
        "Alice should have all 10 token A back after refunding",
    );

    // Check that the offer account is closed
    let offer_account_data = litesvm.get_account(&offer_account);
    assert!(
        offer_account_data.is_none() || offer_account_data.unwrap().data.is_empty(),
        "Offer account should be closed after refund"
    );
}

#[test]
fn test_non_maker_cannot_refund_offer() {
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

    let mint_authority = create_wallet(&mut litesvm, 1_000_000_000);
    let alice = create_wallet(&mut litesvm, 1_000_000_000);
    let bob = create_wallet(&mut litesvm, 1_000_000_000);

    // Create mints
    let token_mint_a = create_token_mint(&mut litesvm, &mint_authority, 9);
    let token_mint_b = create_token_mint(&mut litesvm, &mint_authority, 9);

    // Create token accounts
    let alice_token_account_a = create_associated_token_account(
        &mut litesvm,
        &alice,
        &token_mint_a.pubkey(),
        &mint_authority,
    );

    // Mint tokens to Alice
    let alice_initial_token_a = 10 * TOKEN_A;
    mint_tokens_to_account(
        &mut litesvm,
        &token_mint_a.pubkey(),
        &alice_token_account_a,
        alice_initial_token_a,
        &mint_authority,
    );

    // Alice creates an offer: 3 token A for 2 token B
    let offer_id = 88888u64;
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
    instruction_data.extend_from_slice(&(3 * TOKEN_A).to_le_bytes()); // token_a_offered_amount
    instruction_data.extend_from_slice(&(2 * TOKEN_B).to_le_bytes()); // token_b_wanted_amount
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
    let result = send_transaction_from_instructions(
        &mut litesvm,
        vec![instruction],
        &[&alice],
        &alice.pubkey(),
    );
    assert!(result.is_ok(), "Alice's offer should succeed");

    // Bob tries to refund Alice's offer (should fail)
    let refund_offer_accounts = RefundOfferAccounts {
        token_program: spl_token::ID,
        system_program: anchor_lang::system_program::ID,
        maker: bob.pubkey(),
        token_mint_a: token_mint_a.pubkey(),
        maker_token_account_a: alice_token_account_a,
        offer_account,
        vault,
    };

    let refund_instruction = build_refund_offer_instruction(refund_offer_accounts);
    let result = send_transaction_from_instructions(
        &mut litesvm,
        vec![refund_instruction],
        &[&bob],
        &bob.pubkey(),
    );
    assert!(
        result.is_err(),
        "Non-maker should not be able to refund an offer"
    );

    // Verify that Alice's balance is still the same (offer not refunded)
    assert_token_balance(
        &litesvm,
        &alice_token_account_a,
        7 * TOKEN_A,
        "Alice's balance should remain unchanged after failed refund attempt",
    );

    // Verify that the offer account still exists
    let offer_account_data = litesvm.get_account(&offer_account);
    assert!(
        offer_account_data.is_some() && !offer_account_data.unwrap().data.is_empty(),
        "Offer account should still exist after failed refund attempt"
    );
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

    let mint_authority = create_wallet(&mut litesvm, 1_000_000_000);
    let alice = create_wallet(&mut litesvm, 1_000_000_000);
    let bob = create_wallet(&mut litesvm, 1_000_000_000);

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
    let alice_initial_token_a = 10 * TOKEN_A;
    let bob_initial_token_a = 1;
    let bob_initial_token_b = 1 * TOKEN_B;

    // Create associated token accounts for Alice and Bob for both token mints
    for (owner, mint, _ata) in [
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

    // Mint tokens to the accounts
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
    let large_token_b_amount = 1000 * TOKEN_B; // Much larger than Bob's balance
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
    instruction_data.extend_from_slice(&(1 * TOKEN_A).to_le_bytes());
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
    let result = send_transaction_from_instructions(
        &mut litesvm,
        vec![instruction],
        &[&alice],
        &alice.pubkey(),
    );
    assert!(result.is_ok(), "Alice's offer should succeed");

    // Try to take the offer with Bob who has insufficient token B
    let discriminator_input = b"global:take_offer";
    let instruction_discriminator =
        anchor_lang::solana_program::hash::hash(discriminator_input).to_bytes()[..8].to_vec();
    let instruction_data = instruction_discriminator;
    let account_metas = vec![
        solana_instruction::AccountMeta::new_readonly(spl_associated_token_account::ID, false),
        solana_instruction::AccountMeta::new_readonly(spl_token::ID, false),
        solana_instruction::AccountMeta::new_readonly(anchor_lang::system_program::ID, false),
        solana_instruction::AccountMeta::new(bob.pubkey(), true),
        solana_instruction::AccountMeta::new(alice.pubkey(), false),
        solana_instruction::AccountMeta::new_readonly(token_mint_a.pubkey(), false),
        solana_instruction::AccountMeta::new_readonly(token_mint_b.pubkey(), false),
        solana_instruction::AccountMeta::new(bob_token_account_a, false),
        solana_instruction::AccountMeta::new(bob_token_account_b, false),
        solana_instruction::AccountMeta::new(alice_token_account_b, false),
        solana_instruction::AccountMeta::new(offer_account, false),
        solana_instruction::AccountMeta::new(vault, false),
    ];
    let instruction = solana_instruction::Instruction {
        program_id,
        accounts: account_metas,
        data: instruction_data,
    };
    let result =
        send_transaction_from_instructions(&mut litesvm, vec![instruction], &[&bob], &bob.pubkey());
    assert!(
        result.is_err(),
        "Take offer with insufficient funds should fail"
    );
}
