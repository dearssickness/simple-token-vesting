use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

declare_id!("9Dt3WPawaT6Jf2aTxauKRhsmrBAn84zA3Mi5uitaWZs3");

#[error_code]
pub enum VestingError {
    #[msg("You are not authorized to perform this action.")]
    Unauthorized,
    #[msg("Invalid percentage.")]
    InvalidPercentage,
    #[msg("Nothing available to claim.")]
    NothingToClaim,
    #[msg("Claim before cliff time")]
    EarlyClaim,
    #[msg("Claim after vesting time")]
    LateClaim,
}

#[program]
pub mod simple_token_vesting {

    use super::*;

    pub fn initialize_accounts(_ctx: Context<InitializeAccounts>) -> Result<()> {
        Ok(())
    }
    
    pub fn add_beneficiary(ctx: Context<AddBeneficiary>, total_tokens: u64, beneficiary_wallet: Pubkey) -> Result<()> {
        let beneficiary_data = &mut ctx.accounts.beneficiary_data;
        beneficiary_data.beneficiary_wallet = beneficiary_wallet;
        beneficiary_data.total_tokens = total_tokens;
        beneficiary_data.claimed_tokens = 0;
        Ok(())
    }
    
    pub fn initialize_vesting(
        ctx: Context<InitializeVesting>, 
        amount: u64, decimals: u8, 
        start_time: i64,
        cliff_duration: u64,
        vesting_duration: u64,
    ) -> Result<()> {

        let config = &mut ctx.accounts.config;
        
        config.escrow_wallet = ctx.accounts.escrow_wallet.key();
        config.authority = ctx.accounts.authority.key();
        config.token_mint = ctx.accounts.token_mint.key();
        config.decimals = decimals;
        config.percent_available = 0;
        config.start_time = start_time;
        config.cliff_duration = cliff_duration;
        config.vesting_duration = vesting_duration;

        let token_program = ctx.accounts.token_program.to_account_info();
        
        let transfer = token::Transfer {
            from: ctx.accounts.admin_token_account.to_account_info(),
            to: ctx.accounts.escrow_wallet.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(token_program, transfer);
        
        token::transfer(cpi_ctx, amount * u64::pow(10, decimals as u32))?;
        Ok(())
    }
    
    pub fn release(ctx: Context<Release>, percent: u8) -> Result<()> {
        let config = &mut ctx.accounts.config;

        require!(
            ctx.accounts.authority.key() == config.authority,
            VestingError::Unauthorized
        ); 

        require!(percent <= 100, VestingError::InvalidPercentage);
        config.percent_available = percent;

        Ok(())
    }
    
    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        let beneficiary_data = &mut ctx.accounts.beneficiary_data;
        let beneficiary_wallet = &mut ctx.accounts.beneficiary_wallet;
        let config = &ctx.accounts.config;
        let start_time = config.start_time;
        let cliff_time = start_time + config.cliff_duration as i64;
        let vesting_time = cliff_time + config.vesting_duration as i64;
        
        let clock = Clock::get()?;
        require!(clock.unix_timestamp > cliff_time, VestingError::EarlyClaim);
        
//        let percent_available = if 
//            clock.unix_timestamp >= vesting_time {
//                100
//            } else {
//                config.percent_available
//            };
        
        let percent_available = if clock.unix_timestamp < cliff_time {
            0
        } else if clock.unix_timestamp >= vesting_time {
            100
        } else {
            let elapsed = (clock.unix_timestamp - cliff_time) as u64;
            (elapsed * 100) / config.vesting_duration
        };

        let max_claimable =
            (beneficiary_data.total_tokens * percent_available as u64) / 100;
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
            authority: ctx.accounts.authority.to_account_info()
        };

        let cpi_ctx = CpiContext::new_with_signer (
            ctx.accounts.token_program.to_account_info(),
            transfer,
            signer 
        );
        
        token::transfer (cpi_ctx, claimable_now)?;
        beneficiary_data.claimed_tokens += claimable_now;        

        Ok(())
    }

}

#[derive(Accounts)]
pub struct InitializeAccounts<'info> {
    #[account(
        init,
        seeds = [b"config_vesting", token_mint.key().as_ref()],
        bump,
        payer = user,
        space = 8 + 32 + 32 + 32 + 1 + 1,
    )]
    pub config: Account<'info, ConfigVesting>,
    
    #[account(
        init,
        seeds = [b"escrow", config.key().as_ref()],
        bump,
        payer = user,
        token::mint = token_mint,
        token::authority = authority 
    )]
    pub escrow_wallet: Account<'info, TokenAccount>,
    
    #[account(
        init,
        seeds = [b"beneficiary_data", beneficiary_wallet.key().as_ref()],
        bump,
        payer = user,
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
    pub user: Signer<'info>,

    pub token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,

}

#[derive(Accounts)]
pub struct AddBeneficiary<'info> {
    #[account(
        mut,
        seeds = [b"beneficiary_data", beneficiary_wallet.key().as_ref()],
        bump,
    )]
    pub beneficiary_data: Account<'info, Beneficiary>,

    pub beneficiary_wallet: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

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
    pub user: Signer<'info>,

    #[account(mut)]
    pub admin_token_account: Account<'info, TokenAccount>,
    pub token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,

}

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

#[derive(Accounts)]
pub struct Release<'info>{
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
    pub user: Signer<'info>,
    pub token_mint: Account<'info, Mint>,
}

#[account]
pub struct ConfigVesting {
    pub authority: Pubkey, 
    pub token_mint: Pubkey,
    pub escrow_wallet: Pubkey,
    pub decimals: u8,
    pub percent_available: u8,
    pub start_time: i64,
    pub cliff_duration: u64,
    pub vesting_duration: u64,
}

#[account]
pub struct Beneficiary {
    beneficiary_wallet: Pubkey,
    total_tokens: u64,
    claimed_tokens: u64,

}