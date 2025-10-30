use litesvm::LiteSVM;
use solana_instruction::AccountMeta;
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_kite::{
    create_associated_token_account, create_token_mint, deploy_program, get_pda_and_bump,
    mint_tokens_to_account, send_transaction_from_instructions, SolanaKiteError,
};
use solana_program::hash::hashv;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use std::cell::Cell;
use std::str::FromStr;

pub const PROGRAM_ID: &str = "8jR5GeNzeweq35Uo84kGP3v1NcBaZWH5u62k7PxN4T2y";

/// Standard token unit for token A (1 token = 1_000_000_000 lamports for 9 decimals)
pub const TOKEN_A: u64 = 1_000_000_000;

/// Standard token unit for token B (1 token = 1_000_000_000 lamports for 9 decimals)
pub const TOKEN_B: u64 = 1_000_000_000;

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
    pub _mint_authority: Keypair,
    /// Token mint A (the first token in escrow trades)
    pub token_mint_a: Pubkey,
    /// Token mint B (the second token in escrow trades)
    pub token_mint_b: Pubkey,
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
    deploy_program(&mut litesvm, &program_id, "../../target/deploy/escrow.so").unwrap();

    // Create and fund mint authority
    let mint_authority = Keypair::new();
    litesvm
        .airdrop(&mint_authority.pubkey(), 1_000_000_000)
        .unwrap();

    // Create token mints
    let token_mint_a = create_token_mint(&mut litesvm, &mint_authority, 9, None).unwrap();
    let token_mint_b = create_token_mint(&mut litesvm, &mint_authority, 9, None).unwrap();

    // Create and fund Alice and Bob
    let alice = Keypair::new();
    let bob = Keypair::new();
    litesvm.airdrop(&alice.pubkey(), 1_000_000_000).unwrap();
    litesvm.airdrop(&bob.pubkey(), 1_000_000_000).unwrap();

    // Create associated token accounts for both users and both token types
    let alice_token_account_a = create_associated_token_account(
        &mut litesvm,
        &alice.pubkey(),
        &token_mint_a,
        &mint_authority,
    )
    .unwrap();
    let alice_token_account_b = create_associated_token_account(
        &mut litesvm,
        &alice.pubkey(),
        &token_mint_b,
        &mint_authority,
    )
    .unwrap();
    let bob_token_account_a = create_associated_token_account(
        &mut litesvm,
        &bob.pubkey(),
        &token_mint_a,
        &mint_authority,
    )
    .unwrap();
    let bob_token_account_b = create_associated_token_account(
        &mut litesvm,
        &bob.pubkey(),
        &token_mint_b,
        &mint_authority,
    )
    .unwrap();

    // Mint initial token balances
    // Alice: 10 token A, 0 token B
    // Bob: 0 token A, 5 token B
    mint_tokens_to_account(
        &mut litesvm,
        &token_mint_a,
        &alice_token_account_a,
        10 * TOKEN_A, // Alice gets 10 token A
        &mint_authority,
    )
    .unwrap();
    mint_tokens_to_account(
        &mut litesvm,
        &token_mint_b,
        &bob_token_account_b,
        5 * TOKEN_B, // Bob gets 5 token B
        &mint_authority,
    )
    .unwrap();

    EscrowTestEnvironment {
        litesvm,
        program_id,
        _mint_authority: mint_authority,
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

thread_local! {
    static OFFER_ID_COUNTER: Cell<u64> = Cell::new(1);
}

/// Generates a unique offer ID for testing
///
/// This function returns incrementing offer IDs starting from 1, ensuring
/// each test gets unique IDs to avoid conflicts between test cases.
pub fn generate_offer_id() -> u64 {
    OFFER_ID_COUNTER.with(|counter| {
        let id = counter.get();
        counter.set(id + 1);
        id
    })
}

pub fn get_make_offer_discriminator() -> Vec<u8> {
    let discriminator_input = b"global:make_offer";
    hashv(&[discriminator_input]).to_bytes()[..8].to_vec()
}

pub fn get_take_offer_discriminator() -> Vec<u8> {
    let discriminator_input = b"global:take_offer";
    hashv(&[discriminator_input]).to_bytes()[..8].to_vec()
}

pub fn get_refund_offer_discriminator() -> Vec<u8> {
    let discriminator_input = b"global:refund_offer";
    hashv(&[discriminator_input]).to_bytes()[..8].to_vec()
}

pub fn get_create_game_discriminator() -> Vec<u8> {
    let discriminator_input = b"global:create_game";
    hashv(&[discriminator_input]).to_bytes()[..8].to_vec()
}

pub fn get_deposit_discriminator() -> Vec<u8> {
    let discriminator_input = b"global:deposit";
    hashv(&[discriminator_input]).to_bytes()[..8].to_vec()
}

pub fn get_finalize_game_discriminator() -> Vec<u8> {
    let discriminator_input = b"global:finalize_game";
    hashv(&[discriminator_input]).to_bytes()[..8].to_vec()
}

pub fn get_cancel_game_discriminator() -> Vec<u8> {
    let discriminator_input = b"global:cancel_game";
    hashv(&[discriminator_input]).to_bytes()[..8].to_vec()
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

/// Helper function to create MakeOfferAccounts with standard program IDs
///
/// This function eliminates the repetitive initialization of the three standard
/// program IDs (associated_token_program, token_program, system_program) that
/// are always the same constants across all tests. Instead of copy-pasting
/// these three lines in every test, this helper focuses on the variable fields.
pub fn build_make_offer_accounts(
    maker: Pubkey,
    token_mint_a: Pubkey,
    token_mint_b: Pubkey,
    maker_token_account_a: Pubkey,
    offer_account: Pubkey,
    vault: Pubkey,
) -> MakeOfferAccounts {
    MakeOfferAccounts {
        associated_token_program: anchor_spl::associated_token::ID,
        token_program: anchor_spl::token::ID,
        system_program: anchor_lang::system_program::ID,
        maker,
        token_mint_a,
        token_mint_b,
        maker_token_account_a,
        offer_account,
        vault,
    }
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

pub struct CreateGameAccounts {
    pub authority: Pubkey,
    pub system_program: Pubkey,
    pub game: Pubkey,
}

pub fn build_create_game_instruction(
    id: u64,
    player_a: Pubkey,
    player_b: Pubkey,
    stake_lamports: u64,
    expiry_ts: i64,
    accounts: CreateGameAccounts,
) -> Instruction {
    let mut data = get_create_game_discriminator();
    data.extend_from_slice(&id.to_le_bytes());
    data.extend_from_slice(player_a.as_ref());
    data.extend_from_slice(player_b.as_ref());
    data.extend_from_slice(&stake_lamports.to_le_bytes());
    data.extend_from_slice(&expiry_ts.to_le_bytes());

    let metas = vec![
        AccountMeta::new(accounts.authority, true),
        AccountMeta::new_readonly(accounts.system_program, false),
        AccountMeta::new(accounts.game, false),
    ];

    Instruction { program_id: get_program_id(), accounts: metas, data }
}

pub struct DepositAccounts {
    pub player: Pubkey,
    pub system_program: Pubkey,
    pub game: Pubkey,
}

pub fn build_deposit_instruction(amount: u64, accounts: DepositAccounts) -> Instruction {
    let mut data = get_deposit_discriminator();
    data.extend_from_slice(&amount.to_le_bytes());
    let metas = vec![
        AccountMeta::new(accounts.player, true),
        AccountMeta::new_readonly(accounts.system_program, false),
        AccountMeta::new(accounts.game, false),
    ];
    Instruction { program_id: get_program_id(), accounts: metas, data }
}

pub struct FinalizeGameAccounts {
    pub authority: Pubkey,
    pub winner_account: Pubkey,
    pub system_program: Pubkey,
    pub game: Pubkey,
}

pub fn build_finalize_game_instruction(winner: u8, accounts: FinalizeGameAccounts) -> Instruction {
    let mut data = get_finalize_game_discriminator();
    data.push(winner);
    let metas = vec![
        AccountMeta::new(accounts.authority, true),
        AccountMeta::new(accounts.winner_account, false),
        AccountMeta::new_readonly(accounts.system_program, false),
        AccountMeta::new(accounts.game, false),
    ];
    Instruction { program_id: get_program_id(), accounts: metas, data }
}

pub struct CancelGameAccounts {
    pub caller: Pubkey,
    pub player_a_account: Pubkey,
    pub player_b_account: Pubkey,
    pub system_program: Pubkey,
    pub game: Pubkey,
}

pub fn build_cancel_game_instruction(accounts: CancelGameAccounts) -> Instruction {
    let data = get_cancel_game_discriminator();
    let metas = vec![
        AccountMeta::new(accounts.caller, true),
        AccountMeta::new(accounts.player_a_account, false),
        AccountMeta::new(accounts.player_b_account, false),
        AccountMeta::new_readonly(accounts.system_program, false),
        AccountMeta::new(accounts.game, false),
    ];
    Instruction { program_id: get_program_id(), accounts: metas, data }
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

/// Executes a complete make_offer flow: creates PDAs, builds accounts, and executes instruction
///
/// This helper eliminates the repetitive pattern of creating offer_account and vault PDAs,
/// building MakeOfferAccounts, and executing the make_offer instruction that appears in
/// multiple tests.
pub fn execute_make_offer(
    test_env: &mut EscrowTestEnvironment,
    offer_id: u64,
    maker: &Keypair,
    maker_token_account_a: Pubkey,
    token_a_offered_amount: u64,
    token_b_wanted_amount: u64,
) -> Result<(Pubkey, Pubkey), SolanaKiteError> {
    // Create PDAs
    let (offer_account, _offer_bump) = get_pda_and_bump(
        &[
            b"offer".as_ref().into(),
            offer_id.to_le_bytes().as_ref().into(),
        ],
        &test_env.program_id,
    );
    let vault = anchor_spl::associated_token::get_associated_token_address(
        &offer_account,
        &test_env.token_mint_a,
    );

    // Build accounts
    let make_offer_accounts = build_make_offer_accounts(
        maker.pubkey(),
        test_env.token_mint_a,
        test_env.token_mint_b,
        maker_token_account_a,
        offer_account,
        vault,
    );

    // Build and execute instruction
    let make_offer_instruction = build_make_offer_instruction(
        offer_id,
        token_a_offered_amount,
        token_b_wanted_amount,
        make_offer_accounts,
    );

    send_transaction_from_instructions(
        &mut test_env.litesvm,
        vec![make_offer_instruction],
        &[maker],
        &maker.pubkey(),
    )?;

    Ok((offer_account, vault))
}

/// Executes a complete take_offer flow: builds accounts and executes instruction
pub fn execute_take_offer(
    test_env: &mut EscrowTestEnvironment,
    taker: &Keypair,
    maker: &Keypair,
    taker_token_account_a: Pubkey,
    taker_token_account_b: Pubkey,
    maker_token_account_b: Pubkey,
    offer_account: Pubkey,
    vault: Pubkey,
) -> Result<(), SolanaKiteError> {
    let take_offer_accounts = TakeOfferAccounts {
        associated_token_program: anchor_spl::associated_token::ID,
        token_program: anchor_spl::token::ID,
        system_program: anchor_lang::system_program::ID,
        taker: taker.pubkey(),
        maker: maker.pubkey(),
        token_mint_a: test_env.token_mint_a,
        token_mint_b: test_env.token_mint_b,
        taker_token_account_a,
        taker_token_account_b,
        maker_token_account_b,
        offer_account,
        vault,
    };

    let take_offer_instruction = build_take_offer_instruction(take_offer_accounts);

    send_transaction_from_instructions(
        &mut test_env.litesvm,
        vec![take_offer_instruction],
        &[taker],
        &taker.pubkey(),
    )
}

/// Executes a complete refund_offer flow: builds accounts and executes instruction
pub fn execute_refund_offer(
    test_env: &mut EscrowTestEnvironment,
    maker: &Keypair,
    maker_token_account_a: Pubkey,
    offer_account: Pubkey,
    vault: Pubkey,
) -> Result<(), SolanaKiteError> {
    let refund_offer_accounts = RefundOfferAccounts {
        token_program: anchor_spl::token::ID,
        system_program: anchor_lang::system_program::ID,
        maker: maker.pubkey(),
        token_mint_a: test_env.token_mint_a,
        maker_token_account_a,
        offer_account,
        vault,
    };

    let refund_instruction = build_refund_offer_instruction(refund_offer_accounts);

    send_transaction_from_instructions(
        &mut test_env.litesvm,
        vec![refund_instruction],
        &[maker],
        &maker.pubkey(),
    )
}
