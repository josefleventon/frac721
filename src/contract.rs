use std::any::Any;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    ensure, from_binary, Addr, Binary, QueryResponse, Response, StdError, StdResult, Storage,
    SubMsg, Uint128, WasmMsg,
};
use cw20::MinterResponse;
use cw20_base::msg::{
    InstantiateMarketingInfo, InstantiateMsg as Cw20InstantiateMsg, QueryMsg as Cw20QueryMsg,
};
use cw721::{Cw721QueryMsg, OwnerOfResponse as Cw721OwnerOfResponse};
use cw_storage_plus::{IndexedMap, Item, MultiIndex};
use sylvia::types::{ExecCtx, InstantiateCtx, QueryCtx};
use sylvia::{contract, entry_points};

use crate::error::*;
use crate::msg::*;
use crate::storage::*;

pub struct Frac721Contract {
    pub(crate) collection_address: Item<'static, Addr>,
    pub(crate) cw20_address: Item<'static, Option<Addr>>,
    pub(crate) vault: IndexedMap<'static, &'static str, VaultItem, VaultIndexes<'static>>,
}

#[cw_serde]
pub struct Cw20Config {
    pub code_id: u64,
    pub name: String,
    pub symbol: String,
    pub marketing: Option<InstantiateMarketingInfo>,
}

#[entry_points]
#[contract]
#[error(ContractError)]
impl Frac721Contract {
    pub const fn new() -> Self {
        let indexes = VaultIndexes {
            depositor: MultiIndex::new(
                |_, d| d.depositor.clone(),
                "vault_items",
                "vault_items__depositor",
            ),
        };

        Self {
            collection_address: Item::new("collection_address"),
            cw20_address: Item::new("cw20_address"),
            vault: IndexedMap::new("vault", indexes),
        }
    }

    pub fn must_set_cw20_address(&self, storage: &dyn Storage) -> Result<Addr, ContractError> {
        // Throw an error if the CW20 address has not been set
        let cw20_address = self.cw20_address.load(storage)?;
        ensure!(cw20_address.is_some(), ContractError::Cw20AddressNotSet);
        Ok(cw20_address.unwrap())
    }

    pub fn must_not_set_cw20_address(&self, storage: &dyn Storage) -> Result<(), ContractError> {
        // Throw an error if the CW20 address has already been set
        let cw20_address = self.cw20_address.load(storage)?;
        ensure!(cw20_address.is_none(), ContractError::Cw20AddressAlreadySet);
        Ok(())
    }

    #[msg(instantiate)]
    pub fn instantiate(
        &self,
        context: InstantiateCtx,
        collection_address: String,
        cw20_config: Option<Cw20Config>,
    ) -> StdResult<Response> {
        // Get contract address to pass to CW20 instantiation
        let contract_address = context.env.contract.address;

        let collection_addr = context.deps.api.addr_validate(&collection_address)?;

        // Save the collection address
        self.collection_address
            .save(context.deps.storage, &collection_addr)?;

        // Save the CW20 address as None
        self.cw20_address.save(context.deps.storage, &None)?;

        let response = Response::new()
            .add_attribute("method", "instantiate")
            .add_attribute("contract_address", contract_address.to_string())
            .add_attribute("collection_address", collection_addr.to_string());

        // Instantiate a CW20 contract if config is provided
        // Instantiators may choose not to provide a config and use their own custom CW20
        match cw20_config {
            Some(config) => {
                // Instantiate a CW20 contract
                // We won't have the address until later,
                // so the instantiator will have to execute SetTokenAddress
                let cw20_instantiate_msg = Cw20InstantiateMsg {
                    name: config.name.clone(),
                    symbol: config.symbol.clone(),
                    decimals: 6,
                    initial_balances: vec![],
                    mint: Some(MinterResponse {
                        minter: contract_address.to_string(),
                        cap: None,
                    }),
                    marketing: config.marketing,
                };

                let cw20_msg = match serde_json::to_vec(&cw20_instantiate_msg) {
                    Ok(vec) => Binary::from(vec),
                    Err(e) => {
                        return Err(StdError::generic_err(format!("Serialization error: {}", e)))
                    }
                };

                let instantiate_cw20 = SubMsg::new(WasmMsg::Instantiate {
                    admin: Some(contract_address.to_string()),
                    code_id: config.code_id,
                    funds: vec![],
                    label: config.name.clone(),
                    msg: cw20_msg,
                });

                Ok(response
                    .add_attribute("cw20_name", config.name)
                    .add_attribute("cw20_symbol", config.symbol)
                    .add_attribute("cw20_code_id", config.code_id.to_string())
                    .add_attribute("did_instantiate_cw20", true.to_string())
                    .add_submessage(instantiate_cw20))
            }
            None => Ok(response.add_attribute("did_instantiate_cw20", false.to_string())),
        }
    }

    #[msg(query)]
    pub fn config(&self, context: QueryCtx) -> StdResult<ConfigResponse> {
        let collection_address = self.collection_address.load(context.deps.storage)?;
        let cw20_address = self.cw20_address.load(context.deps.storage)?;

        Ok(ConfigResponse {
            collection_address,
            cw20_address,
        })
    }

    #[msg(exec)]
    pub fn set_token_address(
        &self,
        context: ExecCtx,
        cw20_address: String,
    ) -> Result<Response, ContractError> {
        self.must_not_set_cw20_address(context.deps.storage)?;

        let cw20_addr = context.deps.api.addr_validate(&cw20_address)?;

        // Query the CW20 contract to make sure it's valid
        let cw20_minter_response: MinterResponse = context
            .deps
            .querier
            .query_wasm_smart(cw20_addr.clone(), &Cw20QueryMsg::Minter {})?;

        // Make sure the CW20 contract is mintable by this contract
        assert_eq!(
            cw20_minter_response.minter,
            context.env.contract.address.to_string()
        );

        // Save the CW20 address
        self.cw20_address
            .save(context.deps.storage, &Some(cw20_addr.clone()))?;

        Ok(Response::new()
            .add_attribute("method", "set_cw20_address")
            .add_attribute("contract_address", context.env.contract.address.to_string())
            .add_attribute("cw20_address", cw20_addr.to_string()))
    }

    #[msg(exec)]
    pub fn receive_nft(
        &self,
        context: ExecCtx,
        sender: String,
        token_id: String,
    ) -> Result<Response, ContractError> {
        let cw20_address = self.must_set_cw20_address(context.deps.storage)?;
        let collection_address = self.collection_address.load(context.deps.storage)?;

        // Query the CW721 contract to verify the token ID's owner
        let cw721_owner_response: Cw721OwnerOfResponse = context
            .deps
            .querier
            .query_wasm_smart(
                self.collection_address.load(context.deps.storage)?,
                &Cw721QueryMsg::OwnerOf {
                    token_id: token_id.clone(),
                    include_expired: None,
                },
            )
            .map_err(|error| ContractError::Std(error))?;

        // Make sure the token has been sent to the contract
        ensure!(
            cw721_owner_response.owner == context.env.contract.address.to_string(),
            ContractError::Cw721NotOwnedByContract
        );

        let depositor = context.deps.api.addr_validate(&sender)?;

        // Save the token ID to the vault
        let vault_item = VaultItem {
            token_id: token_id.clone(),
            depositor,
        };
        self.vault
            .save(context.deps.storage, token_id.as_str(), &vault_item)?;

        // Mint 1 CW20 token
        let cw20_mint_msg = cw20::Cw20ExecuteMsg::Mint {
            recipient: sender.clone(),
            amount: Uint128::from(1000000u128),
        };

        let cw20_msg = match serde_json::to_vec(&cw20_mint_msg) {
            Ok(vec) => Binary::from(vec),
            Err(e) => {
                return Err(ContractError::Std(StdError::generic_err(format!(
                    "Serialization error: {}",
                    e
                ))))
            }
        };

        let mint_cw20 = SubMsg::new(WasmMsg::Execute {
            contract_addr: cw20_address.to_string(),
            msg: cw20_msg,
            funds: vec![],
        });

        Ok(Response::new()
            .add_attribute("method", "deposit")
            .add_attribute("contract_address", context.env.contract.address.to_string())
            .add_attribute("collection_address", collection_address.to_string())
            .add_attribute("token_id", token_id)
            .add_attribute("depositor", sender)
            .add_submessage(mint_cw20))
    }

    #[msg(exec)]
    pub fn receive(
        &self,
        context: ExecCtx,
        sender: String,
        amount: Uint128,
        msg: Binary,
    ) -> Result<Response, ContractError> {
        let cw20_address = self.must_set_cw20_address(context.deps.storage)?;
        let collection_address = self.collection_address.load(context.deps.storage)?;

        // Unwrap binary msg object
        let unwrapped_msg: ReceiveMsg =
            from_binary(&msg).map_err(|error| ContractError::Std(error))?;

        // Verify that the correct amount of tokens was sent
        ensure!(
            amount == Uint128::from(1000000u128),
            ContractError::IncorrectTokenAmount
        );

        // Remove the NFT from the vault
        self.vault
            .remove(context.deps.storage, unwrapped_msg.token_id.as_str())?;

        // Send the NFT to the sender
        let cw721_transfer_msg = cw721::Cw721ExecuteMsg::TransferNft {
            recipient: sender.clone(),
            token_id: unwrapped_msg.token_id.clone(),
        };

        let cw721_msg = match serde_json::to_vec(&cw721_transfer_msg) {
            Ok(vec) => Binary::from(vec),
            Err(e) => {
                return Err(ContractError::Std(StdError::generic_err(format!(
                    "Serialization error: {}",
                    e
                ))))
            }
        };

        let transfer_cw721 = SubMsg::new(WasmMsg::Execute {
            contract_addr: collection_address.to_string(),
            msg: cw721_msg,
            funds: vec![],
        });

        // Burn the CW20 token
        let cw20_burn_msg = cw20::Cw20ExecuteMsg::Burn {
            amount: Uint128::from(1000000u128),
        };

        let cw20_msg = match serde_json::to_vec(&cw20_burn_msg) {
            Ok(vec) => Binary::from(vec),
            Err(e) => {
                return Err(ContractError::Std(StdError::generic_err(format!(
                    "Serialization error: {}",
                    e
                ))))
            }
        };

        let burn_cw20 = SubMsg::new(WasmMsg::Execute {
            contract_addr: cw20_address.to_string(),
            msg: cw20_msg,
            funds: vec![],
        });

        Ok(Response::new()
            .add_attribute("method", "claim")
            .add_attribute("contract_address", context.env.contract.address.to_string())
            .add_attribute("collection_address", collection_address.to_string())
            .add_attribute("token_id", unwrapped_msg.token_id)
            .add_attribute("recipient", sender)
            .add_submessage(transfer_cw721)
            .add_submessage(burn_cw20))
    }
}
