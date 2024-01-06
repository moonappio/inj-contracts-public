#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use cw_utils::must_pay;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{SaleConfig, SaleData, State, SALE_CONFIG, SALE_DATA, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:moon-sale";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let state = State {
        owner: info.sender.clone(),
    };
    STATE.save(deps.storage, &state)?;

    SALE_CONFIG.save(deps.storage, &msg.sale)?;

    SALE_DATA.save(deps.storage, &SaleData { total_spent: 0 })?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateSale { sale } => execute::update_sale(deps, info, sale),
        ExecuteMsg::SetUserList { users } => execute::set_user_list(deps, info, users),
        ExecuteMsg::Buy {} => execute::buy(deps, env, info),
        ExecuteMsg::Withdraw { to, funds } => execute::withdraw(deps, info, to, funds),
        ExecuteMsg::TransferOwnership { address } => {
            execute::transfer_ownership(deps, info, address)
        }
    }
}

pub mod execute {
    use cosmwasm_std::{BankMsg, Coin};

    use crate::{
        msg::UserInfoReq,
        state::{UserInfo, USER_LIST},
    };

    use super::*;

    pub fn transfer_ownership(
        deps: DepsMut,
        info: MessageInfo,
        address: String,
    ) -> Result<Response, ContractError> {
        // only owner
        let owner = STATE.load(deps.storage)?.owner;
        if info.sender != owner {
            return Err(ContractError::Unauthorized {});
        }

        let new_owner = deps.api.addr_validate(&address)?;
        STATE.save(
            deps.storage,
            &State {
                owner: new_owner.clone(),
            },
        )?;

        Ok(Response::new()
            .add_attribute("action", "transfer_ownership")
            .add_attribute("from", info.sender)
            .add_attribute("to", new_owner))
    }

    pub fn update_sale(
        deps: DepsMut,
        info: MessageInfo,
        sale: SaleConfig,
    ) -> Result<Response, ContractError> {
        // only owner
        let owner = STATE.load(deps.storage)?.owner;
        if info.sender != owner {
            return Err(ContractError::Unauthorized {});
        }

        SALE_CONFIG.save(deps.storage, &sale)?;
        Ok(Response::new().add_attribute("action", "update_sale"))
    }

    pub fn set_user_list(
        deps: DepsMut,
        info: MessageInfo,
        users: Vec<UserInfoReq>,
    ) -> Result<Response, ContractError> {
        // only owner
        let owner = STATE.load(deps.storage)?.owner;
        if info.sender != owner {
            return Err(ContractError::Unauthorized {});
        }

        for user in users {
            let addr = deps.api.addr_validate(&user.address)?;
            USER_LIST.save(
                deps.storage,
                &addr,
                &UserInfo {
                    allocation: user.allocation,
                    spent: user.spent,
                    received_amount: 0,
                },
            )?;
        }
        Ok(Response::new().add_attribute("action", "set_user_list"))
    }

    // TODO: handle price and add receive_token
    pub fn buy(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        // only if sale is active
        let sale = SALE_CONFIG.load(deps.storage)?;
        if !sale.is_active(env.block.time) {
            return Err(ContractError::SaleNotActive {});
        }

        // only correct funds
        let funds = must_pay(&info, &sale.pay_denom)
            .map_err(|_| ContractError::MissingFunds {})?
            .u128();

        let user = USER_LIST.load(deps.storage, &info.sender)?;

        // only if user is in list and has available allocation
        if user.allocation == 0 {
            return Err(ContractError::NotParticipating {});
        }

        // only if user has enough allocation left
        if funds > user.available_allocation() {
            return Err(ContractError::UserAllocationExceeded {
                wanted: funds,
                max: user.available_allocation(),
            });
        }

        let sale_data = SALE_DATA.load(deps.storage)?;
        let potential_amount: u128 = sale_data.total_spent + funds;

        // only if sale has enough allocation left
        if potential_amount > sale.max_supply {
            return Err(ContractError::SaleAllocationExceeded {
                wanted: funds,
                max: sale.max_supply - sale_data.total_spent,
            });
        }

        let total_tokens = query::receive_amount(deps.as_ref(), funds)?;
        // sucessfull buy
        SALE_DATA.save(
            deps.storage,
            &SaleData {
                total_spent: potential_amount,
            },
        )?;
        USER_LIST.save(
            deps.storage,
            &info.sender,
            &UserInfo {
                allocation: user.allocation,
                spent: user.spent + funds,
                received_amount: user.received_amount + total_tokens,
            },
        )?;

        Ok(Response::new()
            .add_attribute("action", "buy")
            .add_attribute("address", info.sender)
            .add_attribute("amount", funds.to_string())
            .add_attribute("tokens_bought", total_tokens.to_string()))
    }

    pub fn withdraw(
        deps: DepsMut,
        info: MessageInfo,
        to: String,
        funds: Vec<Coin>,
    ) -> Result<Response, ContractError> {
        // only owner
        let owner = STATE.load(deps.storage)?.owner;
        if info.sender != owner {
            return Err(ContractError::Unauthorized {});
        }

        let bank_msg = BankMsg::Send {
            to_address: deps.api.addr_validate(&to)?.to_string(),
            amount: funds,
        };

        Ok(Response::new()
            .add_message(bank_msg)
            .add_attribute("action", "withdraw")
            .add_attribute("to", to))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetSale {} => to_json_binary(&query::sale(deps)?),
        QueryMsg::GetUserInfo { address } => to_json_binary(&query::user(deps, address)?),
        QueryMsg::GetUsers {} => to_json_binary(&query::users(deps)?),
        QueryMsg::GetReceivedAmount { pay_amount } => {
            to_json_binary(&query::receive_amount(deps, pay_amount)?)
        }
    }
}

pub mod query {
    use cosmwasm_std::{Addr, Order};

    use crate::{
        msg::{GetSaleResponse, UserInfoRes},
        state::{UserInfo, USER_LIST},
    };

    use super::*;

    pub fn receive_amount(deps: Deps, pay_amount: u128) -> StdResult<u128> {
        let sale_config = SALE_CONFIG.load(deps.storage)?;

        Ok(pay_amount * sale_config.price_receive_amount / sale_config.price_pay_amount)
    }

    pub fn sale(deps: Deps) -> StdResult<GetSaleResponse> {
        let sale_config = SALE_CONFIG.load(deps.storage)?;
        let sale_data = SALE_DATA.load(deps.storage)?;
        Ok(GetSaleResponse {
            sale_config,
            sale_data,
        })
    }

    pub fn user(deps: Deps, address: String) -> StdResult<UserInfoRes> {
        let user = USER_LIST
            .load(deps.storage, &Addr::unchecked(address.clone()))
            .unwrap_or(UserInfo {
                allocation: 0,
                spent: 0,
                received_amount: 0,
            });

        Ok(UserInfoRes {
            address,
            allocation: user.allocation,
            spent: user.spent,
            received_amount: user.received_amount,
        })
    }

    pub fn users(deps: Deps) -> StdResult<Vec<UserInfoRes>> {
        let users: Vec<UserInfoRes> = USER_LIST
            .range(deps.storage, None, None, Order::Ascending)
            .map(|item| {
                let (k, v) = item.unwrap();
                UserInfoRes {
                    address: k.to_string(),
                    allocation: v.allocation,
                    spent: v.spent,
                    received_amount: v.received_amount,
                }
            })
            .collect();

        Ok(users)
    }
}
