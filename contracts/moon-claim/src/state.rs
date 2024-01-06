use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, StdError, Timestamp};
use cw_storage_plus::{Item, Map};

use crate::ContractError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub owner: Addr,
    pub paused: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ClaimInfo {
    pub reward_denom: String,
    pub initial_unlock: u128, // percentage unlocked at claim_time (100% = 10000)
    pub vesting_start: Timestamp,
    pub vesting_cliff: u128,
    pub vesting_time: u128,
    pub vesting_interval: u128,
}

impl State {
    pub fn check_paused(&self) -> Result<(), ContractError> {
        if self.paused {
            return Err(ContractError::NotActive {});
        }

        Ok(())
    }

    pub fn check_owner(&self, sender: Addr) -> Result<(), ContractError> {
        if sender != self.owner {
            return Err(ContractError::Unauthorized {});
        }

        Ok(())
    }
}

impl ClaimInfo {
    pub fn is_started(&self, time: Timestamp) -> Result<bool, StdError> {
        if time < self.vesting_start {
            return Ok(false);
        }

        Ok(true)
    }

    pub fn unlocked_amount(&self, total_amount: u128, time: Timestamp) -> Result<u128, StdError> {
        if !(self.is_started(time)?) {
            return Ok(0);
        }

        let time_since_claim = (time.seconds() - self.vesting_start.seconds()) as u128;

        let result = if time_since_claim <= self.vesting_cliff {
            // initial unlock only
            (total_amount * self.initial_unlock) / 10000
        } else if time_since_claim > (self.vesting_cliff + self.vesting_time) {
            // vesting finished
            total_amount
        } else {
            let initial_amount = (total_amount * self.initial_unlock) / 10000;
            let vestable = total_amount - initial_amount;

            let intervals_since = (time_since_claim - self.vesting_cliff) / self.vesting_interval;
            let total_vesting_intervals = self.vesting_time / self.vesting_interval;

            (vestable * intervals_since) / total_vesting_intervals + initial_amount
        };

        Ok(result)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct UserInfo {
    pub reward: u128,
    pub withdrawn: u128,
}

pub const STATE: Item<State> = Item::new("state");
pub const CLAIM_INFO: Item<ClaimInfo> = Item::new("claim_info");
pub const USER_INFO: Map<&Addr, UserInfo> = Map::new("users");
