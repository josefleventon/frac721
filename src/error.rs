use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("CW20 contract address has not been set.")]
    Cw20AddressNotSet,

    #[error("CW20 contract address has already been set.")]
    Cw20AddressAlreadySet,

    #[error("Token not sent to contract.")]
    Cw721NotOwnedByContract,

    #[error("Incorrect token amount sent to contract.")]
    IncorrectTokenAmount,
}
