use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::*;

#[derive(Accounts)]
pub struct InitializeAccounts<'info> {
    #[account(
        init,
        seeds = [b"config_vesting", token_mint.key().as_ref()],
        bump,
        payer = admin,
        space = 8 + 32 + 32 + 32 + 32 + 1 + 1 + 8 + 8 + 8 + 1 + 1,
    )]
    pub config: Account<'info, ConfigVesting>,
    
    #[account(
        init,
        seeds = [b"escrow", config.key().as_ref()],
        bump,
        payer = admin,
        token::mint = token_mint,
        token::authority = authority 
    )]
    pub escrow_wallet: Account<'info, TokenAccount>,
    
    #[account(
        init,
        seeds = [b"beneficiary_data", beneficiary_wallet.key().as_ref()],
        bump,
        payer = admin,
        space = 8 + 32 + 8 + 8,
    )]
    pub beneficiary_data: Account<'info, Beneficiary>,

    #[account(mut)]
    pub beneficiary_wallet: Account<'info, TokenAccount>,

    /// CHECK: This PDA is used only as a signing authority, no data is read or written.
    #[account(
        seeds = [b"authority", token_mint.key().as_ref()],
        bump,
    )]
    pub authority: AccountInfo<'info>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(_ctx: Context<InitializeAccounts>) -> Result<()> {
    Ok(())
}