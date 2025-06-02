use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use crate::{errors::VestingError, state::*};

#[derive(Accounts)]
pub struct RevokeVesting<'info> {
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
}

pub fn handler(ctx: Context<RevokeVesting>) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.vesting_revoked = true;
    
    let token_key = &ctx.accounts.token_mint.key();
    let authority = &ctx.accounts.authority;

    let authority_seeds = &[
        b"authority".as_ref(),
        token_key.as_ref(),
        &[ctx.bumps.authority],
    ];
    
    let signer = &[&authority_seeds[..]];

    require!(
        ctx.accounts.admin.key() == config.admin,
        VestingError::Unauthorized
    );

    let escrow_wallet = &mut ctx.accounts.escrow_wallet;
    let admin_token_account = &mut ctx.accounts.admin_token_account;
    
    let transfer = token::Transfer {
        from: escrow_wallet.to_account_info(),
        to: admin_token_account.to_account_info(),
        authority: authority.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        transfer,
        signer,
    );

    require!(escrow_wallet.amount > 0, VestingError::NothingToReclaim);
    token::transfer(cpi_ctx, escrow_wallet.amount)?;

    Ok(())
}