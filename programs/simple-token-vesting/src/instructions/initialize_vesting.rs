use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use crate::state::*;

#[derive(Accounts)]
pub struct InitializeVesting<'info> {
    #[account(
        mut,
        seeds = [b"config_vesting", token_mint.key().as_ref()],
        bump,
    )]
    pub config: Account<'info, ConfigVesting>,
    
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
    pub admin: Signer<'info>,

    #[account(mut)]
    pub admin_token_account: Account<'info, TokenAccount>,
    pub token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    }

pub fn handler(
    ctx: Context<InitializeVesting>,
    amount: u64,
    decimals: u8,
    start_time: i64,
    cliff_duration: u64,
    vesting_duration: u64,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    
    config.escrow_wallet = ctx.accounts.escrow_wallet.key();
    config.admin = ctx.accounts.admin.key();
    config.authority = ctx.accounts.authority.key();
    config.token_mint = ctx.accounts.token_mint.key();
    config.decimals = decimals;
    config.percent_available = 0;
    config.start_time = start_time;
    config.cliff_duration = cliff_duration;
    config.vesting_duration = vesting_duration;
    config.vesting_revoked = false;
    config.auto_vesting = false;

    let token_program = ctx.accounts.token_program.to_account_info();
    
    let transfer = token::Transfer {
        from: ctx.accounts.admin_token_account.to_account_info(),
        to: ctx.accounts.escrow_wallet.to_account_info(),
        authority: ctx.accounts.admin.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(token_program, transfer);
    
    token::transfer(cpi_ctx, amount * u64::pow(10, decimals as u32))?;
    Ok(())
}