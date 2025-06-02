use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use crate::{errors::VestingError, state::*};

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(
        mut,
        seeds = [b"config_vesting", token_mint.key().as_ref()],
        bump,
    )]
    pub config: Account<'info, ConfigVesting>,

    #[account(
        mut,
        seeds = [b"beneficiary_data", beneficiary_wallet.key().as_ref()],
        bump,
    )]
    pub beneficiary_data: Account<'info, Beneficiary>,

    #[account(mut)]
    pub beneficiary_wallet: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"escrow", config.key().as_ref()],
        bump,
        token::mint = token_mint,
        token::authority = authority
    )]
    pub escrow_wallet: Account<'info, TokenAccount>,
    
    /// CHECK: This PDA is used only as a signing authority, no data is read or written.
    #[account(
        seeds = [b"authority", token_mint.key().as_ref()],
        bump,
    )]
    pub authority: AccountInfo<'info>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Claim>) -> Result<()> {
    let beneficiary_data = &mut ctx.accounts.beneficiary_data;
    let beneficiary_wallet = &mut ctx.accounts.beneficiary_wallet;
    let config = &ctx.accounts.config;
    let vesting_revoked = config.vesting_revoked;
    let percent_available = config.percent_available;
    let cliff_time = config.start_time + config.cliff_duration as i64;
    
    require!(!vesting_revoked, VestingError::VestingRevoked);

    let clock = Clock::get()?;
    require!(clock.unix_timestamp > cliff_time, VestingError::EarlyClaim);
    
    let max_claimable = (beneficiary_data.total_tokens * percent_available as u64) / 100;
    let claimable_now = max_claimable.saturating_sub(beneficiary_data.claimed_tokens);
    
    require!(claimable_now > 0, VestingError::NothingToClaim);

    let token_key = &ctx.accounts.token_mint.key();

    let authority_seeds = &[
        b"authority".as_ref(),
        token_key.as_ref(),
        &[ctx.bumps.authority],
    ];
    
    let signer = &[&authority_seeds[..]];

    let transfer = token::Transfer {
        from: ctx.accounts.escrow_wallet.to_account_info(),
        to: beneficiary_wallet.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        transfer,
        signer,
    );
    
    token::transfer(cpi_ctx, claimable_now)?;
    beneficiary_data.claimed_tokens += claimable_now;        

    Ok(())
}