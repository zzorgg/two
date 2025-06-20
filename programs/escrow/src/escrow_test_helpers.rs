use crate::test_helpers::{
    create_associated_token_account, create_token_mint, deploy_program, mint_tokens_to_account,
};
use litesvm::LiteSVM;
use solana_instruction::AccountMeta;
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use std::str::FromStr;

pub const PROGRAM_ID: &str = "8jR5GeNzeweq35Uo84kGP3v1NcBaZWH5u62k7PxN4T2y";

/// Standard token unit (1 token = 1_000_000_000 lamports for 9 decimals)
pub const TOKEN: u64 = 1_000_000_000;

/// Complete escrow test environment containing all necessary components for testing
///
/// This struct holds all the accounts, keypairs, and state needed for comprehensive
/// escrow testing scenarios. It's returned by `setup_escrow_test()` and provides
/// a convenient way to access all test components.
pub struct EscrowTestEnvironment {
    /// The LiteSVM instance for simulating Solana transactions
    pub litesvm: LiteSVM,
    /// The escrow program ID
    pub program_id: Pubkey,
    /// The mint authority that can create and mint tokens
    pub mint_authority: Keypair,
    /// Token mint A (the first token in escrow trades)
    pub token_mint_a: Keypair,
    /// Token mint B (the second token in escrow trades)
    pub token_mint_b: Keypair,
    /// Alice's keypair (typically the offer maker)
    pub alice: Keypair,
    /// Bob's keypair (typically the offer taker)
    pub bob: Keypair,
    /// Alice's token account for token A
    pub alice_token_account_a: Pubkey,
    /// Alice's token account for token B
    pub alice_token_account_b: Pubkey,
    /// Bob's token account for token A
    pub bob_token_account_a: Pubkey,
    /// Bob's token account for token B
    pub bob_token_account_b: Pubkey,
}

/// Sets up a complete escrow test environment with all necessary components
///
/// This function performs the following setup steps:
/// 1. Creates a new LiteSVM instance for transaction simulation
/// 2. Deploys the escrow program to the test environment
/// 3. Creates a mint authority and funds it with SOL
/// 4. Creates two token mints (A and B) with 9 decimals
/// 5. Creates Alice and Bob keypairs and funds them with SOL
/// 6. Creates associated token accounts for both users and both token types
/// 7. Mints initial token balances:
///    - Alice: 10 token A, 0 token B
///    - Bob: 0 token A, 5 token B
///
/// # Returns
///
/// Returns an `EscrowTestEnvironment` struct containing all the test components
/// needed for escrow testing scenarios.
///
/// # Example
///
/// ```rust
/// let env = setup_escrow_test();
///
/// // Create an offer using Alice
/// let offer_id = 12345u64;
/// let (offer_account, _) = Pubkey::find_program_address(
///     &[b"offer", &offer_id.to_le_bytes()],
///     &env.program_id
/// );
///
/// // Use the environment components for testing
/// let make_offer_accounts = MakeOfferAccounts {
///     associated_token_program: spl_associated_token_account::ID,
///     token_program: spl_token::ID,
///     system_program: anchor_lang::system_program::ID,
///     maker: env.alice.pubkey(),
///     token_mint_a: env.token_mint_a.pubkey(),
///     token_mint_b: env.token_mint_b.pubkey(),
///     maker_token_account_a: env.alice_token_account_a,
///     offer_account,
///     vault: spl_associated_token_account::get_associated_token_address(&offer_account, &env.token_mint_a.pubkey()),
/// };
/// ```
pub fn setup_escrow_test() -> EscrowTestEnvironment {
    let mut litesvm = LiteSVM::new();
    let program_id = get_program_id();

    // Deploy the escrow program
    deploy_program(&mut litesvm, &program_id, "../../target/deploy/escrow.so");

    // Create and fund mint authority
    let mint_authority = Keypair::new();
    litesvm
        .airdrop(&mint_authority.pubkey(), 1_000_000_000)
        .unwrap();

    // Create token mints
    let token_mint_a = create_token_mint(&mut litesvm, &mint_authority, 9);
    let token_mint_b = create_token_mint(&mut litesvm, &mint_authority, 9);

    // Create and fund Alice and Bob
    let alice = Keypair::new();
    let bob = Keypair::new();
    litesvm.airdrop(&alice.pubkey(), 1_000_000_000).unwrap();
    litesvm.airdrop(&bob.pubkey(), 1_000_000_000).unwrap();

    // Create associated token accounts for both users and both token types
    let alice_token_account_a = create_associated_token_account(
        &mut litesvm,
        &alice,
        &token_mint_a.pubkey(),
        &mint_authority,
    );
    let alice_token_account_b = create_associated_token_account(
        &mut litesvm,
        &alice,
        &token_mint_b.pubkey(),
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

    // Mint initial token balances
    // Alice: 10 token A, 0 token B
    // Bob: 0 token A, 5 token B
    mint_tokens_to_account(
        &mut litesvm,
        &token_mint_a.pubkey(),
        &alice_token_account_a,
        10 * TOKEN, // Alice gets 10 token A
        &mint_authority,
    );
    mint_tokens_to_account(
        &mut litesvm,
        &token_mint_b.pubkey(),
        &bob_token_account_b,
        5 * TOKEN, // Bob gets 5 token B
        &mint_authority,
    );

    EscrowTestEnvironment {
        litesvm,
        program_id,
        mint_authority,
        token_mint_a,
        token_mint_b,
        alice,
        bob,
        alice_token_account_a,
        alice_token_account_b,
        bob_token_account_a,
        bob_token_account_b,
    }
}

pub fn get_program_id() -> Pubkey {
    Pubkey::from_str(PROGRAM_ID).unwrap()
}

pub fn get_make_offer_discriminator() -> Vec<u8> {
    let discriminator_input = b"global:make_offer";
    anchor_lang::solana_program::hash::hash(discriminator_input).to_bytes()[..8].to_vec()
}

pub fn get_take_offer_discriminator() -> Vec<u8> {
    let discriminator_input = b"global:take_offer";
    anchor_lang::solana_program::hash::hash(discriminator_input).to_bytes()[..8].to_vec()
}

pub fn get_refund_offer_discriminator() -> Vec<u8> {
    let discriminator_input = b"global:refund_offer";
    anchor_lang::solana_program::hash::hash(discriminator_input).to_bytes()[..8].to_vec()
}

pub struct MakeOfferAccounts {
    pub associated_token_program: Pubkey,
    pub token_program: Pubkey,
    pub system_program: Pubkey,
    pub maker: Pubkey,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub maker_token_account_a: Pubkey,
    pub offer_account: Pubkey,
    pub vault: Pubkey,
}

pub fn build_make_offer_instruction(
    offer_id: u64,
    token_a_offered_amount: u64,
    token_b_wanted_amount: u64,
    accounts: MakeOfferAccounts,
) -> Instruction {
    let mut instruction_data = get_make_offer_discriminator();
    instruction_data.extend_from_slice(&offer_id.to_le_bytes());
    instruction_data.extend_from_slice(&token_a_offered_amount.to_le_bytes());
    instruction_data.extend_from_slice(&token_b_wanted_amount.to_le_bytes());

    let account_metas = vec![
        AccountMeta::new_readonly(accounts.associated_token_program, false),
        AccountMeta::new_readonly(accounts.token_program, false),
        AccountMeta::new_readonly(accounts.system_program, false),
        AccountMeta::new(accounts.maker, true),
        AccountMeta::new_readonly(accounts.token_mint_a, false),
        AccountMeta::new_readonly(accounts.token_mint_b, false),
        AccountMeta::new(accounts.maker_token_account_a, false),
        AccountMeta::new(accounts.offer_account, false),
        AccountMeta::new(accounts.vault, false),
    ];

    Instruction {
        program_id: get_program_id(),
        accounts: account_metas,
        data: instruction_data,
    }
}

pub struct TakeOfferAccounts {
    pub associated_token_program: Pubkey,
    pub token_program: Pubkey,
    pub system_program: Pubkey,
    pub taker: Pubkey,
    pub maker: Pubkey,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub taker_token_account_a: Pubkey,
    pub taker_token_account_b: Pubkey,
    pub maker_token_account_b: Pubkey,
    pub offer_account: Pubkey,
    pub vault: Pubkey,
}

pub fn build_take_offer_instruction(accounts: TakeOfferAccounts) -> Instruction {
    let instruction_data = get_take_offer_discriminator();

    let account_metas = vec![
        AccountMeta::new_readonly(accounts.associated_token_program, false),
        AccountMeta::new_readonly(accounts.token_program, false),
        AccountMeta::new_readonly(accounts.system_program, false),
        AccountMeta::new(accounts.taker, true),
        AccountMeta::new(accounts.maker, false),
        AccountMeta::new_readonly(accounts.token_mint_a, false),
        AccountMeta::new_readonly(accounts.token_mint_b, false),
        AccountMeta::new(accounts.taker_token_account_a, false),
        AccountMeta::new(accounts.taker_token_account_b, false),
        AccountMeta::new(accounts.maker_token_account_b, false),
        AccountMeta::new(accounts.offer_account, false),
        AccountMeta::new(accounts.vault, false),
    ];

    Instruction {
        program_id: get_program_id(),
        accounts: account_metas,
        data: instruction_data,
    }
}

pub struct RefundOfferAccounts {
    pub token_program: Pubkey,
    pub system_program: Pubkey,
    pub maker: Pubkey,
    pub token_mint_a: Pubkey,
    pub maker_token_account_a: Pubkey,
    pub offer_account: Pubkey,
    pub vault: Pubkey,
}

pub fn build_refund_offer_instruction(accounts: RefundOfferAccounts) -> Instruction {
    let instruction_data = get_refund_offer_discriminator();

    let account_metas = vec![
        AccountMeta::new_readonly(accounts.token_program, false),
        AccountMeta::new_readonly(accounts.system_program, false),
        AccountMeta::new(accounts.maker, true),
        AccountMeta::new_readonly(accounts.token_mint_a, false),
        AccountMeta::new(accounts.maker_token_account_a, false),
        AccountMeta::new(accounts.offer_account, false),
        AccountMeta::new(accounts.vault, false),
    ];

    Instruction {
        program_id: get_program_id(),
        accounts: account_metas,
        data: instruction_data,
    }
}
