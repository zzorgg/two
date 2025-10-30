use anchor_lang::prelude::*;

use crate::error::ErrorCode;
use crate::handlers::transfer_lamports;
use crate::state::Game;

#[derive(Accounts)]
pub struct FinalizeGame<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: paid as a SystemAccount; we validate against game players
    #[account(mut)]
    pub winner_account: AccountInfo<'info>,

    pub system_program: Program<'info, System>,

    #[account(
        mut,
        close = authority,
        seeds = [b"game", game.id.to_le_bytes().as_ref()],
        bump = game.bump,
    )]
    pub game: Account<'info, Game>,
}

pub fn finalize_game(ctx: Context<FinalizeGame>, winner: u8) -> Result<()> {
    let game = &mut ctx.accounts.game;

    require!(ctx.accounts.authority.key() == game.authority, ErrorCode::Unauthorized);
    require!(game.a_deposited && game.b_deposited, ErrorCode::DepositsIncomplete);

    // Determine winner pubkey
    let expected_winner = match winner {
        1 => game.player_a,
        2 => game.player_b,
        _ => return Err(error!(ErrorCode::InvalidWinner)),
    };
    require!(ctx.accounts.winner_account.key() == expected_winner, ErrorCode::InvalidWinner);

    // Transfer total stake to winner from the game PDA
    let total = game
        .stake_lamports
        .checked_mul(2)
        .ok_or(error!(ErrorCode::LamportsTransferFailed))?;

    let seeds = [b"game".as_ref(), &game.id.to_le_bytes()[..], &[game.bump]];
    transfer_lamports(
        &game.to_account_info(),
        &ctx.accounts.winner_account,
        total,
        &ctx.accounts.system_program,
        Some(&seeds),
    )
    .map_err(|_| ErrorCode::LamportsTransferFailed)?;

    game.winner = winner;

    Ok(())
}
