use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub owner: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct SaleConfig {
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub pay_denom: String,
    pub sale_denom: Option<String>,
    pub max_supply: u128,
    pub price_pay_amount: u128,
    pub price_receive_amount: u128,
}

impl SaleConfig {
    pub fn is_active(&self, time: Timestamp) -> bool {
        self.start_time <= time && time < self.end_time
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserInfo {
    pub allocation: u128,
    pub spent: u128,
    pub received_amount: u128,
}

impl UserInfo {
    pub fn available_allocation(&self) -> u128 {
        self.allocation - self.spent
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SaleData {
    pub total_spent: u128,
}

pub const STATE: Item<State> = Item::new("state");
pub const SALE_CONFIG: Item<SaleConfig> = Item::new("sale");
pub const USER_LIST: Map<&Addr, UserInfo> = Map::new("users");
pub const SALE_DATA: Item<SaleData> = Item::new("sale_data");
