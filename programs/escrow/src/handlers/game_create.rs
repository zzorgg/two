use anchor_lang::prelude::*;

use crate::state::Game;
use crate::error::ErrorCode;

#[derive(Accounts)]
#[instruction(id: u64)]
pub struct CreateGame<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(
        init,
        payer = authority,
        space = Game::DISCRIMINATOR.len() + Game::INIT_SPACE,
        seeds = [b"game", id.to_le_bytes().as_ref()],
        bump
    )]
    pub game: Account<'info, Game>,
}

pub fn create_game(
    ctx: Context<CreateGame>,
    id: u64,
    player_a: Pubkey,
    player_b: Pubkey,
    stake_lamports: u64,
    expiry_ts: i64,
) -> Result<()> {
    require!(player_a != player_b, ErrorCode::Unauthorized);
    require!(stake_lamports > 0, ErrorCode::InvalidAmount);

    let now = Clock::get()?.unix_timestamp;
    require!(expiry_ts > now, ErrorCode::InvalidAmount);

    let bump = ctx.bumps.game;
    ctx.accounts.game.set_inner(Game {
        id,
        player_a,
        player_b,
        authority: ctx.accounts.authority.key(),
        stake_lamports,
        a_deposited: false,
        b_deposited: false,
        winner: 0,
        expiry_ts,
        bump,
    });

    Ok(())
}
