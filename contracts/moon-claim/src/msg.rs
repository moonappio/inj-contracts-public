use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::state::{ClaimInfo, State, UserInfo};

#[cw_serde]
pub struct UserInfoItem {
    pub address: String,
    pub user_info: UserInfo,
}
#[cw_serde]
pub struct InstantiateMsg {
    pub claim_info: ClaimInfo,
}

#[cw_serde]
pub enum ExecuteMsg {
    TogglePause {},
    TransferOwnership {
        address: String,
    },
    UpdateClaimInfo {
        claim_info: ClaimInfo,
    },
    Claim {},
    SetUsers {
        users: Vec<UserInfoItem>,
    },
    Withdraw {
        address: String,
        denom: String,
        amount: u128,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(State)]
    GetState {},

    #[returns(ClaimInfo)]
    GetClaim {},

    #[returns(u128)]
    TotalAvailableAfter {},

    #[returns(u128)]
    GetWithdrawableAmount { address: String },

    #[returns(UserInfo)]
    GetUserInfo { address: String },

    #[returns(Vec<UserInfoItem>)]
    GetUsers {},
}
