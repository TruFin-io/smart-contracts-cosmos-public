use cosmwasm_std::{ConversionOverflowError, DivideByZeroError, StdError};
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Only the owner can call this method")]
    OnlyOwner,

    #[error("There is no pending owner set")]
    NoPendingOwnerSet,

    #[error("Only the pending owner can call this method")]
    NotPendingOwner,

    #[error("Fee cannot be larger than fee precision")]
    FeeTooLarge,

    #[error("Minimum deposit amount is too small")]
    MinimumDepositTooSmall,

    #[error("Deposit amount is below the min deposit amount")]
    DepositBelowMinDeposit,

    #[error("Insufficient INJ attached")]
    InsufficientInjAttached,

    #[error("Duplicate initial balance addresses")]
    DuplicateInitialBalanceAddresses,

    #[error("Cannot set to own account")]
    CannotSetOwnAccount,

    #[error("Payment error: {0}")]
    Payment(#[from] PaymentError),

    #[error("Divide by zero error: {0}")]
    ZeroDiv(#[from] DivideByZeroError),

    #[error("Overflow: {0}")]
    Overflow(#[from] ConversionOverflowError),

    #[error("Validator already exists")]
    ValidatorAlreadyExists,

    #[error("Validator does not exist")]
    ValidatorDoesNotExist,

    #[error("Validator is already enabled")]
    ValidatorAlreadyEnabled,

    #[error("Validator is already disabled")]
    ValidatorAlreadyDisabled,

    #[error("Validator is disabled")]
    ValidatorNotEnabled,

    // Whitelist Errors
    #[error("Caller is not an agent")]
    CallerIsNotAgent,

    #[error("Ower cannot be added")]
    OwnerCannotBeAdded,

    #[error("Owner cannot be removed")]
    OwnerCannotBeRemoved,

    #[error("Agent already exists")]
    AgentAlreadyExists,

    #[error("Agent does not exist")]
    AgentDoesNotExist,

    #[error("User already whitelisted")]
    UserAlreadyWhitelisted,

    #[error("User already blacklisted")]
    UserAlreadyBlacklisted,

    #[error("User status already cleared")]
    UserStatusAlreadyCleared,

    #[error("User not whitelisted")]
    UserNotWhitelisted,

    #[error("Contract is paused")]
    ContractPaused,

    #[error("Contract is not paused")]
    NotPaused,

    #[error("Insufficient TruINJ balance")]
    InsufficientTruINJBalance,

    #[error("Unstake amount too low")]
    UnstakeAmountTooLow,

    #[error("Redelegate amount too low")]
    RedelegateAmountTooLow,

    #[error("Shares amount too low")]
    SharesAmountTooLow,

    #[error("Insufficient funds on validator")]
    InsufficientValidatorFunds,

    #[error("Insufficient funds on staker")]
    InsufficientStakerFunds,

    #[error("No withdrawals to claim")]
    NothingToClaim,

    #[error("Validator is not in validator set")]
    NotInValidatorSet,
}

impl From<cw20_base::ContractError> for ContractError {
    fn from(err: cw20_base::ContractError) -> Self {
        match err {
            cw20_base::ContractError::Std(error) => Self::Std(error),
            cw20_base::ContractError::Unauthorized {} => Self::Unauthorized,
            cw20_base::ContractError::CannotSetOwnAccount {} => Self::CannotSetOwnAccount,
            cw20_base::ContractError::DuplicateInitialBalanceAddresses {} => {
                Self::DuplicateInitialBalanceAddresses {}
            }
            _ => Self::Std(StdError::generic_err(err.to_string())),
        }
    }
}
