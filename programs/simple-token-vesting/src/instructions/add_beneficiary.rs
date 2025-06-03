use anchor_lang::prelude::*;
use anchor_spl::token::{TokenAccount};
use crate::state::*;

#[derive(Accounts)]
pub struct AddBeneficiary<'info> {
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

    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<AddBeneficiary>,
    total_tokens: u64,
    beneficiary_wallet: Pubkey,
) -> Result<()> {
    let beneficiary_data = &mut ctx.accounts.beneficiary_data;
    beneficiary_data.beneficiary_wallet = beneficiary_wallet;
    beneficiary_data.total_tokens = total_tokens;
    beneficiary_data.claimed_tokens = 0;
    Ok(())
}