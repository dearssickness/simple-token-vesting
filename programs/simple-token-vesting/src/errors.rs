use anchor_lang::prelude::*;

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
    #[msg("Vesting revoked by admin")]
    VestingRevoked,
    #[msg("Escrow wallet is empty")]
    NothingToReclaim,
}