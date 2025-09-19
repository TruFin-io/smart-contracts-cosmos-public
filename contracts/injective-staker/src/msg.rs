use crate::state::{UserStatus, ValidatorInfo};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Attribute, Binary, Uint128, Uint256};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub treasury: String,
    pub default_validator: String,
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    SetFee {
        new_fee: u16,
    },
    SetMinimumDeposit {
        new_min_deposit: Uint128,
    },
    SetTreasury {
        new_treasury_addr: String,
    },
    SetDefaultValidator {
        new_default_validator_addr: String,
    },
    /// Transfer is a base message to move tokens to another account without triggering actions
    Transfer {
        recipient: String,
        amount: Uint128,
    },
    /// Send is a base message to transfer tokens to a contract and trigger an action
    /// on the receiving contract. Receiving contract must implement receiver interface.
    Send {
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
    Stake {},
    StakeToSpecificValidator {
        validator_addr: String,
    },
    Unstake {
        amount: Uint128,
    },
    UnstakeFromSpecificValidator {
        validator_addr: String,
        amount: Uint128,
    },
    Redelegate {
        src_validator_addr: String,
        dst_validator_addr: String,
        assets: Uint128,
    },
    Claim {},
    AddValidator {
        validator: String,
    },
    EnableValidator {
        validator: String,
    },
    DisableValidator {
        validator: String,
    },
    // Whitelist messages
    AddAgent {
        agent: String,
    },
    RemoveAgent {
        agent: String,
    },
    SetPendingOwner {
        new_owner: String,
    },
    ClaimOwnership {},
    AddUserToWhitelist {
        user: String,
    },
    AddUserToBlacklist {
        user: String,
    },
    ClearUserStatus {
        user: String,
    },
    Pause,
    Unpause,
    CompoundRewards,
    // Internal messages
    Restake {
        amount: Uint128,
        validator_addr: String,
    },
    EmitEvent {
        attributes: Vec<Attribute>,
    },
    #[cfg(any(test, feature = "test"))]
    TestMint {
        recipient: cosmwasm_std::Addr,
        amount: Uint128,
    },
    #[cfg(any(test, feature = "test"))]
    TestSetMinimumDeposit {
        new_min_deposit: Uint128,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetStakerInfoResponse)]
    GetStakerInfo {},
    #[returns(cw20::BalanceResponse)]
    Balance { address: String },
    #[returns(cw20::TokenInfoResponse)]
    TokenInfo {},
    #[returns(cw20::MarketingInfoResponse)]
    MarketingInfo {},
    #[returns(GetIsOwnerResponse)]
    IsOwner { addr: String },
    #[returns(GetValidatorResponse)]
    GetValidators {},
    #[returns(GetTotalStakedResponse)]
    GetTotalStaked {},
    #[returns(GetTotalRewardsResponse)]
    GetTotalRewards {},
    #[returns(GetTotalSupplyResponse)]
    GetTotalSupply {},
    #[returns(GetClaimableAmountResponse)]
    GetClaimableAmount { user: String },

    // Whitelist queries
    #[returns(GetIsAgentResponse)]
    IsAgent { agent: String },
    #[returns(GetIsWhitelistedResponse)]
    IsWhitelisted { user: String },
    #[returns(GetIsBlacklistedResponse)]
    IsBlacklisted { user: String },
    #[returns(GetCurrentUserStatusResponse)]
    GetCurrentUserStatus { user: String },
    #[returns(GetSharePriceResponse)]
    GetSharePrice {},
    #[returns(GetTotalAssetsResponse)]
    GetTotalAssets {},
    #[returns(cw_controllers::ClaimsResponse)]
    GetClaimableAssets { user: String },
    #[returns(GetMaxWithdrawResponse)]
    GetMaxWithdraw { user: String },
}

#[cw_serde]
pub struct GetStakerInfoResponse {
    pub owner: String,
    pub default_validator: String,
    pub treasury: String,
    pub fee: u16,
    pub min_deposit: Uint128,
    pub is_paused: bool,
}

#[cw_serde]
pub struct GetValidatorResponse {
    pub validators: Vec<ValidatorInfo>,
}

#[cw_serde]
pub struct GetIsAgentResponse {
    pub is_agent: bool,
}

#[cw_serde]
pub struct GetIsOwnerResponse {
    pub is_owner: bool,
}

#[cw_serde]
pub struct GetIsWhitelistedResponse {
    pub is_whitelisted: bool,
}

#[cw_serde]
pub struct GetIsBlacklistedResponse {
    pub is_blacklisted: bool,
}

#[cw_serde]
pub struct GetCurrentUserStatusResponse {
    pub user_status: UserStatus,
}

#[cw_serde]
pub struct GetTotalStakedResponse {
    pub total_staked: Uint128,
}

#[cw_serde]
pub struct GetTotalRewardsResponse {
    pub total_rewards: Uint128,
}

#[cw_serde]
pub struct GetTotalSupplyResponse {
    pub total_supply: Uint128,
}

#[cw_serde]
pub struct GetSharePriceResponse {
    pub numerator: Uint256,
    pub denominator: Uint256,
}

#[cw_serde]
pub struct GetTotalAssetsResponse {
    pub total_assets: Uint128,
}

#[cw_serde]
pub struct GetMaxWithdrawResponse {
    pub max_withdraw: Uint128,
}

#[cw_serde]
pub struct GetClaimableAmountResponse {
    pub claimable_amount: Uint128,
}
