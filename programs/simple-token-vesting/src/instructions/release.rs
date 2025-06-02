use anchor_lang::prelude::*;
use anchor_spl::token::{Mint};
use crate::{errors::VestingError, state::*};

#[derive(Accounts)]
pub struct Release<'info> {
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
    pub admin: Signer<'info>,
    pub token_mint: Account<'info, Mint>,
}

pub fn handler(
    ctx: Context<Release>,
    percent: u8,
    auto_vesting: bool,
    vesting_revoked: bool,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    let start_time = config.start_time;
    let cliff_time = start_time + config.cliff_duration as i64;
    let vesting_time = cliff_time + config.vesting_duration as i64;

    require!(!vesting_revoked, VestingError::VestingRevoked);
    require!(
        ctx.accounts.admin.key() == config.admin,
        VestingError::Unauthorized
    );
    require!(percent <= 100, VestingError::InvalidPercentage);

    let clock = Clock::get()?;
    require!(clock.unix_timestamp > cliff_time, VestingError::EarlyClaim);
    
    let percent_available = match (auto_vesting, clock.unix_timestamp) {
        (_, t) if t >= vesting_time => 100,
        (false, _) => percent,
        (true, t) if t < cliff_time => 0,
        (true, t) => {
            let elapsed = (t - cliff_time) as u64;
            ((elapsed * 100) / config.vesting_duration) as u8
        }
    };

    config.percent_available = percent_available;
    Ok(())
}