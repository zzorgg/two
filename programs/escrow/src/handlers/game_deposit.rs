use anchor_lang::prelude::*;

use crate::error::ErrorCode;
use crate::handlers::transfer_lamports;
use crate::state::Game;

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(
        mut,
        seeds = [b"game", game.id.to_le_bytes().as_ref()],
        bump = game.bump,
    )]
    pub game: Account<'info, Game>,
}

pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    let game = &mut ctx.accounts.game;
    require!(amount == game.stake_lamports, ErrorCode::StakeAmountMismatch);

    if ctx.accounts.player.key() == game.player_a {
        require!(!game.a_deposited, ErrorCode::AlreadyDeposited);
        transfer_lamports(
            &ctx.accounts.player.to_account_info(),
            &game.to_account_info(),
            amount,
            &ctx.accounts.system_program,
            None,
        )
        .map_err(|_| ErrorCode::LamportsTransferFailed)?;
        game.a_deposited = true;
    } else if ctx.accounts.player.key() == game.player_b {
        require!(!game.b_deposited, ErrorCode::AlreadyDeposited);
        transfer_lamports(
            &ctx.accounts.player.to_account_info(),
            &game.to_account_info(),
            amount,
            &ctx.accounts.system_program,
            None,
        )
        .map_err(|_| ErrorCode::LamportsTransferFailed)?;
        game.b_deposited = true;
    } else {
        return Err(error!(ErrorCode::Unauthorized));
    }

    Ok(())
}
