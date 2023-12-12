use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub struct ReceiveMsg {
    pub token_id: String,
}

#[cw_serde]
pub struct ConfigResponse {
    pub collection_address: Addr,
    pub cw20_address: Option<Addr>,
}
