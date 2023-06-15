use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
        ConvertCw20ToCoin { token: String, amount: Uint128, recipient: String},
        ConvertCoinToCw20 { token: String, amount: Uint128, recipient: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
