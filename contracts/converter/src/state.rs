use cosmwasm_schema::cw_serde;
use cw_storage_plus::Map;
#[cw_serde]
#[derive(Default)]

pub struct Denom {
    pub token: String,
    pub denom: String,
}

pub const DENOMSDATA: Map<&str, Denom> = Map::new("denoms");