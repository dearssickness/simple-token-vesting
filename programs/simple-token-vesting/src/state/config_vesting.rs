use anchor_lang::prelude::*;

#[account]
pub struct ConfigVesting {
    pub authority: Pubkey,
    pub admin: Pubkey,
    pub token_mint: Pubkey,
    pub escrow_wallet: Pubkey,
    pub decimals: u8,
    pub percent_available: u8,
    pub start_time: i64,
    pub cliff_duration: u64,
    pub vesting_duration: u64,
    pub vesting_invoked: bool,
    pub auto_vesting: bool,
}