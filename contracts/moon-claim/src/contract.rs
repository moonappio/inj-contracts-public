#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{ClaimInfo, State, CLAIM_INFO, STATE, USER_INFO};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:moon-claim";
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
        paused: false,
    };
    STATE.save(deps.storage, &state)?;

    CLAIM_INFO.save(deps.storage, &msg.claim_info)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", state.owner)
        .add_attribute("reward_denom", &msg.claim_info.reward_denom))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::TransferOwnership { address } => {
            execute::transfer_ownership(deps, info, address)
        }
        ExecuteMsg::TogglePause {} => execute::toggle_pause(deps, info),
        ExecuteMsg::UpdateClaimInfo { claim_info } => {
            execute::update_claim_info(deps, info, claim_info)
        }
        ExecuteMsg::Claim {} => execute::claim(deps, _env, info),
        ExecuteMsg::SetUsers { users } => execute::set_users(deps, info, users),
        ExecuteMsg::Withdraw {
            address,
            denom,
            amount,
        } => execute::withdraw(deps, info, address, denom, amount),
    }
}

pub mod execute {
    use cosmwasm_std::{coins, BankMsg};

    use crate::{
        msg::UserInfoItem,
        state::{UserInfo, USER_INFO},
    };

    use super::*;

    pub fn transfer_ownership(
        deps: DepsMut,
        info: MessageInfo,
        address: String,
    ) -> Result<Response, ContractError> {
        // only owner
        let state = STATE.load(deps.storage)?;
        state.check_owner(info.sender.clone())?;

        let new_owner = deps.api.addr_validate(&address)?;
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            state.owner = new_owner.clone();
            Ok(state)
        })?;

        Ok(Response::new()
            .add_attribute("action", "transfer_ownership")
            .add_attribute("from", info.sender.clone())
            .add_attribute("to", new_owner))
    }

    pub fn withdraw(
        deps: DepsMut,
        info: MessageInfo,
        to: String,
        denom: String,
        amount: u128,
    ) -> Result<Response, ContractError> {
        // only owner
        let state = STATE.load(deps.storage)?;
        state.check_owner(info.sender.clone())?;

        let to = deps.api.addr_validate(&to)?;
        let transfer = BankMsg::Send {
            to_address: to.to_string(),
            amount: coins(amount, denom),
        };

        Ok(Response::new()
            .add_message(transfer)
            .add_attribute("action", "withdraw"))
    }

    pub fn toggle_pause(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        // only owner
        let state = STATE.load(deps.storage)?;
        state.check_owner(info.sender)?;

        let new_state = STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            state.paused = !state.paused;
            Ok(state)
        })?;

        Ok(Response::new()
            .add_attribute("action", "toggle_pause")
            .add_attribute("paused", new_state.paused.to_string()))
    }

    pub fn update_claim_info(
        deps: DepsMut,
        info: MessageInfo,
        claim_info: ClaimInfo,
    ) -> Result<Response, ContractError> {
        // only owner
        let state = STATE.load(deps.storage)?;
        state.check_owner(info.sender)?;

        CLAIM_INFO.save(deps.storage, &claim_info)?;

        Ok(Response::new().add_attribute("action", "update_claim_info"))
    }

    pub fn claim(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let state = STATE.load(deps.storage)?;
        state.check_paused()?;

        let claim_info = CLAIM_INFO.load(deps.storage)?;

        if !claim_info.is_started(env.block.time)? {
            return Err(ContractError::NotActive {});
        }

        let withdraw_amount =
            query::get_withdrawable_amount(deps.as_ref(), env, info.sender.to_string())?;

        USER_INFO.update(
            deps.storage,
            &info.sender,
            |state| -> Result<_, ContractError> {
                let mut state = state.unwrap();
                state.withdrawn += withdraw_amount;
                Ok(state)
            },
        )?;

        let transfer = BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: coins(withdraw_amount, claim_info.reward_denom),
        };

        Ok(Response::new()
            .add_message(transfer)
            .add_attribute("action", "claim")
            .add_attribute("amount", withdraw_amount.to_string()))
    }

    pub fn set_users(
        deps: DepsMut,
        info: MessageInfo,
        users: Vec<UserInfoItem>,
    ) -> Result<Response, ContractError> {
        // only owner
        let state = STATE.load(deps.storage)?;
        state.check_owner(info.sender)?;

        for user in users {
            let addr = deps.api.addr_validate(&user.address)?;
            USER_INFO.save(
                deps.storage,
                &addr,
                &UserInfo {
                    reward: user.user_info.reward,
                    withdrawn: user.user_info.withdrawn,
                },
            )?;
        }

        Ok(Response::new().add_attribute("action", "set_users"))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetState {} => to_json_binary(&STATE.load(deps.storage)?),
        QueryMsg::GetClaim {} => to_json_binary(&CLAIM_INFO.load(deps.storage)?),
        QueryMsg::TotalAvailableAfter {} => to_json_binary(&query::total_available_after(deps)?),
        QueryMsg::GetWithdrawableAmount { address } => {
            to_json_binary(&query::get_withdrawable_amount(deps, env, address)?)
        }
        QueryMsg::GetUserInfo { address } => {
            to_json_binary(&USER_INFO.load(deps.storage, &deps.api.addr_validate(&address)?)?)
        }
        QueryMsg::GetUsers {} => to_json_binary(&query::get_users(deps)?),
    }
}

pub mod query {
    use super::*;
    use crate::{msg::UserInfoItem, state::USER_INFO};

    pub fn get_users(deps: Deps) -> StdResult<Vec<UserInfoItem>> {
        let users: Vec<UserInfoItem> = USER_INFO
            .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .map(|item| {
                let (k, v) = item?;
                Ok(UserInfoItem {
                    address: k.to_string(),
                    user_info: v,
                })
            })
            .collect::<StdResult<Vec<UserInfoItem>>>()?;

        Ok(users)
    }

    pub fn total_available_after(deps: Deps) -> StdResult<u128> {
        let claim_info = CLAIM_INFO.load(deps.storage)?;

        Ok(claim_info.vesting_start.seconds() as u128
            + claim_info.vesting_cliff
            + claim_info.vesting_time)
    }

    pub fn get_withdrawable_amount(deps: Deps, env: Env, address: String) -> StdResult<u128> {
        let user_info = USER_INFO.load(deps.storage, &deps.api.addr_validate(&address)?)?;
        let claim_info = CLAIM_INFO.load(deps.storage)?;

        let unlocked_amount = claim_info.unlocked_amount(user_info.reward, env.block.time)?;

        Ok(unlocked_amount - user_info.withdrawn)
    }
}
