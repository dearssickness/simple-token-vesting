#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

declare_id!("9Dt3WPawaT6Jf2aTxauKRhsmrBAn84zA3Mi5uitaWZs3");

pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

#[program]
pub mod simple_token_vesting {
    use super::*;

    pub fn initialize_accounts(ctx: Context<InitializeAccounts>) -> Result<()> {
        instructions::initialize_accounts::handler(ctx)
    }

    pub fn add_beneficiary(
        ctx: Context<AddBeneficiary>,
        total_tokens: u64,
        beneficiary_wallet: Pubkey,
    ) -> Result<()> {
        instructions::add_beneficiary::handler(ctx, total_tokens, beneficiary_wallet)
    }

    pub fn initialize_vesting(
        ctx: Context<InitializeVesting>,
        amount: u64,
        decimals: u8,
        start_time: i64,
        cliff_duration: u64,
        vesting_duration: u64,
    ) -> Result<()> {
        instructions::initialize_vesting::handler(
            ctx,
            amount,
            decimals,
            start_time,
            cliff_duration,
            vesting_duration,
        )
    }

    pub fn revoke_vesting(ctx: Context<RevokeVesting>) -> Result<()> {
        instructions::revoke_vesting::handler(ctx)
    }

    pub fn reconfigure_vesting(
        ctx: Context<Reconfigure>,
        auto_vesting: bool,
        vesting_revoked: bool,
    ) -> Result<()> {
        instructions::reconfigure::handler(ctx, auto_vesting, vesting_revoked)
    }

    pub fn release(
        ctx: Context<Release>,
        percent: u8,
        auto_vesting: bool,
        vesting_revoked: bool,
    ) -> Result<()> {
        instructions::release::handler(ctx, percent, auto_vesting, vesting_revoked)
    }

    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        instructions::claim::handler(ctx)
    }
}