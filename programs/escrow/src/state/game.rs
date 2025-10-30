use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Game {
    // Identifier for the game
    pub id: u64,
    // Both players' wallets
    pub player_a: Pubkey,
    pub player_b: Pubkey,
    // Authority that can finalize/cancel (could be your backend/referee)
    pub authority: Pubkey,
    // Stake in lamports per player
    pub stake_lamports: u64,
    // True once each player deposits
    pub a_deposited: bool,
    pub b_deposited: bool,
    // 0 = none, 1 = A wins, 2 = B wins
    pub winner: u8,
    // Unix timestamp after which timeout cancel can be executed
    pub expiry_ts: i64,
    // Bump for PDA
    pub bump: u8,
}
