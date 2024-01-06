use cw_multi_test::{App, ContractWrapper, Executor, SudoMsg};
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    contract::{execute, instantiate, query},
    msg::{QueryMsg, UserInfoItem},
};
use cosmwasm_std::{Addr, Coin, StdResult};

use crate::{
    msg::{ExecuteMsg, InstantiateMsg},
    state::ClaimInfo,
    ContractError,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MoonClaimContract(pub Addr);

impl MoonClaimContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    // General
    #[track_caller]
    pub fn query_value<T>(&self, app: &App, msg: &QueryMsg) -> StdResult<T>
    where
        T: DeserializeOwned,
    {
        app.wrap().query_wasm_smart(self.0.clone(), msg)
    }

    #[track_caller]
    pub fn mint_coins(app: &mut App, to: &Addr, funds: Vec<Coin>) {
        app.sudo(SudoMsg::Bank(cw_multi_test::BankSudo::Mint {
            to_address: to.to_string(),
            amount: funds,
        }))
        .unwrap();
    }

    // Contract functions
    #[track_caller]
    pub fn store_code(app: &mut App) -> u64 {
        let contract = ContractWrapper::new(execute, instantiate, query);
        app.store_code(Box::new(contract))
    }

    #[track_caller]
    pub fn instantiate(
        app: &mut App,
        code_id: u64,
        sender: &Addr,
        label: &str,
        claim_config: ClaimInfo,
    ) -> StdResult<Self> {
        app.instantiate_contract(
            code_id,
            sender.clone(),
            &InstantiateMsg {
                claim_info: claim_config,
            },
            &[],
            label,
            None,
        )
        .map(MoonClaimContract)
        .map_err(|err| err.downcast().unwrap())
    }

    #[track_caller]
    pub fn claim(&self, app: &mut App, sender: &Addr) -> Result<(), ContractError> {
        app.execute_contract(sender.clone(), self.0.clone(), &ExecuteMsg::Claim {}, &[])
            .map_err(|err| {
                let res = err.downcast();
                match res {
                    Ok(err) => err,
                    Err(err) => ContractError::Std(err.downcast().unwrap()),
                }
            })
            .map(|_| ())
    }

    #[track_caller]
    pub fn withdraw(
        &self,
        app: &mut App,
        sender: &Addr,
        address: String,
        denom: String,
        amount: u128,
    ) -> Result<(), ContractError> {
        app.execute_contract(
            sender.clone(),
            self.0.clone(),
            &ExecuteMsg::Withdraw {
                address,
                denom,
                amount,
            },
            &[],
        )
        .map_err(|err| {
            let res = err.downcast();
            match res {
                Ok(err) => err,
                Err(err) => ContractError::Std(err.downcast().unwrap()),
            }
        })
        .map(|_| ())
    }

    #[track_caller]
    pub fn set_users(
        &self,
        app: &mut App,
        sender: &Addr,
        users: Vec<UserInfoItem>,
    ) -> Result<(), ContractError> {
        app.execute_contract(
            sender.clone(),
            self.0.clone(),
            &ExecuteMsg::SetUsers { users },
            &[],
        )
        .map_err(|err| {
            let res = err.downcast();
            match res {
                Ok(err) => err,
                Err(err) => ContractError::Std(err.downcast().unwrap()),
            }
        })
        .map(|_| ())
    }
}
