use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Sale not active")]
    SaleNotActive {},

    #[error("Not enough funds")]
    MissingFunds {},

    #[error("Not participating")]
    NotParticipating {},

    #[error("User allocation exceeded")]
    UserAllocationExceeded { wanted: u128, max: u128 },

    #[error("Sale allocation exceeded")]
    SaleAllocationExceeded { wanted: u128, max: u128 },
}
