use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Index, IndexList, MultiIndex};

#[cw_serde]
pub struct VaultItem {
    pub token_id: String,
    pub depositor: Addr,
}

pub struct VaultIndexes<'a> {
    pub depositor: MultiIndex<'a, Addr, VaultItem, String>,
}

impl<'a> IndexList<VaultItem> for VaultIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<VaultItem>> + '_> {
        let v: Vec<&dyn Index<VaultItem>> = vec![&self.depositor];
        Box::new(v.into_iter())
    }
}
