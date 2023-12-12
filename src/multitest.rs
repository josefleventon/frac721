#![cfg(test)]

use cosmwasm_std::{coins, Addr, Empty, Uint128};
use cw20::MinterResponse;
use cw721_base::msg::ExecuteMsg as Cw721ExecuteMsg;
use cw_multi_test::{App, Contract, ContractWrapper, Executor};

pub fn contract_cw721() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw721_base::entry::execute,
        cw721_base::entry::instantiate,
        cw721_base::entry::query,
    );
    Box::new(contract)
}

pub fn contract_cw20() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    );
    Box::new(contract)
}

pub fn contract_frac721() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::entry_points::execute,
        crate::contract::entry_points::instantiate,
        crate::contract::entry_points::query,
    );
    Box::new(contract)
}

const CW721: &str = "contract0";
const CW20: &str = "contract1";
const FRAC721: &str = "contract2";

const ADMIN: &str = "admin";
const USER: &str = "user";

// Initial contract setup
fn setup_contracts() -> App {
    let admin = Addr::unchecked(ADMIN);

    let init_funds = coins(2000, "ustars");

    let mut router = App::new(|router, _, storage| {
        router
            .bank
            .init_balance(storage, &admin, init_funds)
            .unwrap();
    });

    // Set up CW721 contract
    let cw721_id = router.store_code(contract_cw721());
    let msg = cw721_base::msg::InstantiateMsg {
        name: String::from("Bad Kids"),
        symbol: String::from("BAD"),
        minter: admin.to_string(),
    };

    let cw721_addr = router
        .instantiate_contract(cw721_id, admin.clone(), &msg, &[], "BAD_CW721", None)
        .unwrap();

    // Set up CW20 contract
    let cw20_id = router.store_code(contract_cw20());
    let msg = cw20_base::msg::InstantiateMsg {
        name: String::from("Fractionalized Bad Kids"),
        symbol: String::from("BAD"),
        decimals: 6,
        initial_balances: vec![],
        mint: Some(MinterResponse {
            minter: FRAC721.to_string(),
            cap: None,
        }),
        marketing: None,
    };
    let cw20_addr = router
        .instantiate_contract(cw20_id, admin.clone(), &msg, &[], "BAD_CW20", None)
        .unwrap();

    // Set up Frac721 contract
    let frac721_id = router.store_code(contract_frac721());
    let msg = crate::contract::InstantiateMsg {
        collection_address: cw721_addr.to_string(),
        cw20_config: None,
    };

    let frac721_addr = router
        .instantiate_contract(frac721_id, admin.clone(), &msg, &[], "BAD_FRAC721", None)
        .unwrap();

    // Execute set_cw20_address on Frac721
    let msg = crate::contract::ExecMsg::SetTokenAddress {
        cw20_address: cw20_addr.to_string(),
    };

    router
        .execute_contract(admin.clone(), frac721_addr, &msg, &[])
        .unwrap();

    router
}

// Mint a CW721 NFT to an address
fn mint_cw721(router: &mut App, addr: Addr, token_id: &str) {
    let msg: Cw721ExecuteMsg<Empty, Empty> = Cw721ExecuteMsg::Mint {
        token_id: token_id.to_string(),
        owner: addr.to_string(),
        token_uri: None,
        extension: Empty {},
    };

    router
        .execute_contract(Addr::unchecked(ADMIN), Addr::unchecked(CW721), &msg, &[])
        .unwrap();
}

// Send a CW721 NFT to a contract
fn send_cw721(router: &mut App, sender: Addr, recipient: Addr, token_id: &str) {
    let msg: Cw721ExecuteMsg<Empty, Empty> = Cw721ExecuteMsg::SendNft {
        contract: recipient.to_string(),
        token_id: token_id.to_string(),
        msg: b"{}".to_vec().into(),
    };

    router
        .execute_contract(sender, Addr::unchecked(CW721), &msg, &[])
        .unwrap();
}

#[test]
fn proper_initialization() {
    setup_contracts();
}

#[test]
fn try_query_config() {
    let router = setup_contracts();
    let msg = crate::contract::QueryMsg::Config {};
    let res: crate::msg::ConfigResponse = router.wrap().query_wasm_smart(FRAC721, &msg).unwrap();
    assert_eq!(res.collection_address, CW721.to_string());
    assert_eq!(res.cw20_address.unwrap(), CW20.to_string());
}

#[test]
fn try_deposit() {
    let mut router = setup_contracts();

    let user = Addr::unchecked(USER);
    let contract = Addr::unchecked(FRAC721);
    let token_id = "1";

    // Mint a CW721 NFT to the user
    mint_cw721(&mut router, user.clone(), token_id);

    // Send the NFT to the Frac721 contract
    send_cw721(&mut router, user.clone(), contract, token_id);

    // Query the CW20 balance of the user
    let msg = cw20_base::msg::QueryMsg::Balance {
        address: user.to_string(),
    };

    let res: cw20::BalanceResponse = router.wrap().query_wasm_smart(CW20, &msg).unwrap();
    assert_eq!(res.balance, Uint128::new(1000000u128));
}

#[test]
fn try_claim() {
    let mut router = setup_contracts();

    let user = Addr::unchecked(USER);
    let contract = Addr::unchecked(FRAC721);
    let token_id = "1";

    // Mint a CW721 NFT to the user
    mint_cw721(&mut router, user.clone(), token_id);

    // Send the NFT to the Frac721 contract
    send_cw721(&mut router, user.clone(), contract.clone(), token_id);

    // Query the CW20 balance of the user
    let msg = cw20_base::msg::QueryMsg::Balance {
        address: user.to_string(),
    };

    let res: cw20::BalanceResponse = router.wrap().query_wasm_smart(CW20, &msg).unwrap();
    assert_eq!(res.balance, Uint128::new(1000000u128));

    // Send a CW20 token to the contract
    let msg = cw20_base::msg::ExecuteMsg::Send {
        contract: contract.to_string(),
        amount: Uint128::new(1000000u128),
        msg: b"{\"token_id\":\"1\"}".to_vec().into(),
    };

    router
        .execute_contract(user.clone(), Addr::unchecked(CW20), &msg, &[])
        .unwrap();

    // Query the CW20 balance of the user
    let msg = cw20_base::msg::QueryMsg::Balance {
        address: user.to_string(),
    };

    let res: cw20::BalanceResponse = router.wrap().query_wasm_smart(CW20, &msg).unwrap();
    assert_eq!(res.balance, Uint128::new(0u128));

    // Verify that the user now owns the NFT
    let msg: cw721_base::QueryMsg<Empty> = cw721_base::msg::QueryMsg::OwnerOf {
        token_id: token_id.to_string(),
        include_expired: None,
    };

    let res: cw721::OwnerOfResponse = router.wrap().query_wasm_smart(CW721, &msg).unwrap();
    assert_eq!(res.owner, user.to_string());
}
