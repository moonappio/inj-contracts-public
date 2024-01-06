use crate::state::{SaleConfig, SaleData, UserInfo};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Coin;

#[cw_serde]
pub struct InstantiateMsg {
    pub sale: SaleConfig,
}

#[cw_serde]
pub struct UserInfoReq {
    pub address: String,
    pub allocation: u128,
    pub spent: u128,
}

#[cw_serde]
pub struct UserInfoRes {
    pub address: String,
    pub allocation: u128,
    pub spent: u128,
    pub received_amount: u128,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateSale { sale: SaleConfig },
    SetUserList { users: Vec<UserInfoReq> },
    Buy {},
    Withdraw { to: String, funds: Vec<Coin> },
    TransferOwnership { address: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetSaleResponse)]
    GetSale {},

    #[returns(UserInfoRes)]
    GetUserInfo { address: String },

    #[returns(Vec<UserInfoRes>)]
    GetUsers {},

    #[returns(u128)]
    GetReceivedAmount { pay_amount: u128 },
}

#[cw_serde]
pub struct GetSaleResponse {
    pub sale_config: SaleConfig,
    pub sale_data: SaleData,
}

#[cw_serde]
pub struct GetUserInfoResponse {
    pub user: UserInfo,
}
