use anchor_lang::prelude::*;
use anchor_spl::token::{Mint};
use crate::{errors::VestingError, state::*};

#[derive(Accounts)]
pub struct Reconfigure<'info> {
    #[account(
        mut,
        seeds = [b"config_vesting", token_mint.key().as_ref()],
        bump,
    )]
    pub config: Account<'info, ConfigVesting>,

    /// CHECK: This PDA is used only as a signing authority, no data is read or written.
    #[account(
        seeds = [b"authority", token_mint.key().as_ref()],
        bump,
    )]
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub token_mint: Account<'info, Mint>,
}

pub fn handler(
    ctx: Context<Reconfigure>,
    auto_vesting: bool,
    vesting_invoked: bool,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    
    require!(
        ctx.accounts.admin.key() == config.admin,
        VestingError::Unauthorized
    );
    
    config.auto_vesting = auto_vesting;
    config.vesting_invoked = vesting_invoked;
    Ok(())
}