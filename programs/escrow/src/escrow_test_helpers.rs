use solana_instruction::AccountMeta;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;
use std::str::FromStr;

pub const PROGRAM_ID: &str = "8jR5GeNzeweq35Uo84kGP3v1NcBaZWH5u62k7PxN4T2y";

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
