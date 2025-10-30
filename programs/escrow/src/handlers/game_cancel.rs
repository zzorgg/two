use anchor_lang::prelude::*;

use crate::error::ErrorCode;
use crate::handlers::transfer_lamports;
use crate::state::Game;

#[derive(Accounts)]
pub struct CancelGame<'info> {
    #[account(mut)]
    pub caller: Signer<'info>,

    /// CHECK: validated against game.player_a
    #[account(mut)]
    pub player_a_account: AccountInfo<'info>,

    /// CHECK: validated against game.player_b
    #[account(mut)]
    pub player_b_account: AccountInfo<'info>,

    pub system_program: Program<'info, System>,

    #[account(
        mut,
        close = caller,
        seeds = [b"game", game.id.to_le_bytes().as_ref()],
        bump = game.bump,
    )]
    pub game: Account<'info, Game>,
}

pub fn cancel_game(ctx: Context<CancelGame>) -> Result<()> {
    let game = &mut ctx.accounts.game;

    // Allow cancel by authority or either player after expiry
    let now = Clock::get()?.unix_timestamp;
    require!(now > game.expiry_ts, ErrorCode::NotExpired);

    let caller = ctx.accounts.caller.key();
    require!(
        caller == game.authority || caller == game.player_a || caller == game.player_b,
        ErrorCode::Unauthorized
    );

    // Validate destination accounts
    require!(ctx.accounts.player_a_account.key() == game.player_a, ErrorCode::Unauthorized);
    require!(ctx.accounts.player_b_account.key() == game.player_b, ErrorCode::Unauthorized);

    let seeds = [b"game".as_ref(), &game.id.to_le_bytes()[..], &[game.bump]];

    // Refund deposited stakes
    if game.a_deposited {
        transfer_lamports(
            &game.to_account_info(),
            &ctx.accounts.player_a_account,
            game.stake_lamports,
            &ctx.accounts.system_program,
            Some(&seeds),
        )
        .map_err(|_| ErrorCode::LamportsTransferFailed)?;
        game.a_deposited = false;
    }

    if game.b_deposited {
        transfer_lamports(
            &game.to_account_info(),
            &ctx.accounts.player_b_account,
            game.stake_lamports,
            &ctx.accounts.system_program,
            Some(&seeds),
        )
        .map_err(|_| ErrorCode::LamportsTransferFailed)?;
        game.b_deposited = false;
    }

    game.winner = 0;

    Ok(())
}
