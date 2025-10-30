use anchor_lang::prelude::*;
use handlers::*;

pub mod constants;
pub mod error;
pub mod handlers;
pub mod state;

declare_id!("8jR5GeNzeweq35Uo84kGP3v1NcBaZWH5u62k7PxN4T2y");

#[program]
pub mod escrow {
    use super::*;

    pub fn make_offer(
        context: Context<MakeOffer>,
        id: u64,
        token_a_offered_amount: u64,
        token_b_wanted_amount: u64,
    ) -> Result<()> {
        handlers::make_offer::make_offer(context, id, token_a_offered_amount, token_b_wanted_amount)
    }

    pub fn take_offer(context: Context<TakeOffer>) -> Result<()> {
        handlers::take_offer::take_offer(context)
    }

    pub fn refund_offer(context: Context<RefundOffer>) -> Result<()> {
        handlers::refund_offer::refund_offer(context)
    }

    // Native SOL duel escrow instructions
    pub fn create_game(
        context: Context<CreateGame>,
        id: u64,
        player_a: Pubkey,
        player_b: Pubkey,
        stake_lamports: u64,
        expiry_ts: i64,
    ) -> Result<()> {
        handlers::game_create::create_game(context, id, player_a, player_b, stake_lamports, expiry_ts)
    }

    pub fn deposit(context: Context<Deposit>, amount: u64) -> Result<()> {
        handlers::game_deposit::deposit(context, amount)
    }

    pub fn finalize_game(context: Context<FinalizeGame>, winner: u8) -> Result<()> {
        handlers::game_finalize::finalize_game(context, winner)
    }

    pub fn cancel_game(context: Context<CancelGame>) -> Result<()> {
        handlers::game_cancel::cancel_game(context)
    }
}

#[cfg(test)]
mod escrow_test_helpers;
#[cfg(test)]
mod tests;
