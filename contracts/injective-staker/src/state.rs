use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw20::Expiration;
use cw_controllers::Claims;
use cw_storage_plus::{Item, Map};
use std::fmt;

#[cw_serde]
pub struct StakerInfo {
    pub treasury: Addr,
    pub fee: u16,
    pub min_deposit: u128,
}

#[cw_serde]
pub struct ValidatorInfo {
    pub total_staked: Uint128,
    pub state: ValidatorState,
    pub addr: String,
}

#[cw_serde]
pub enum ValidatorState {
    None,
    Enabled,
    Disabled,
}

#[cw_serde]
pub struct StakerInfoV1 {
    pub treasury: Addr,
    pub fee: u16,
    pub min_deposit: u128,
    pub distribution_fee: u16,
}

pub const STAKER_INFO: Item<StakerInfo> = Item::new("staker_info");
pub const VALIDATORS: Map<&String, ValidatorState> = Map::new("validators");
pub const DEFAULT_VALIDATOR: Item<String> = Item::new("default_validator");
pub const WHITELIST_AGENTS: Map<&Addr, ()> = Map::new("whitelist_agents");
pub const OWNER: Item<Addr> = Item::new("owner");
pub const PENDING_OWNER: Item<Addr> = Item::new("pending_owner");
pub const WHITELIST_USERS: Map<&Addr, UserStatus> = Map::new("whitelist_users");
pub const IS_PAUSED: Item<bool> = Item::new("is_paused");
pub const CONTRACT_REWARDS: Item<Uint128> = Item::new("contract_rewards");
pub const CLAIMS: Claims = Claims::new("claims");

#[cw_serde]
pub enum UserStatus {
    NoStatus,
    Whitelisted,
    Blacklisted,
}

/// Implement Display for UserStatus
impl fmt::Display for UserStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let status_str = match self {
            Self::NoStatus => "no_status",
            Self::Whitelisted => "whitelisted",
            Self::Blacklisted => "blacklisted",
        };
        write!(f, "{}", status_str)
    }
}

pub trait GetValueTrait {
    fn get_value(&self) -> u64;
}
impl GetValueTrait for Expiration {
    fn get_value(&self) -> u64 {
        match self {
            Self::AtHeight(height) => *height,
            Self::AtTime(time) => time.seconds(),
            Self::Never {} => 0,
        }
    }
}
