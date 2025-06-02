use anchor_lang::prelude::*;

#[account]
pub struct Beneficiary {
    pub beneficiary_wallet: Pubkey,
    pub total_tokens: u64,
    pub claimed_tokens: u64,
}