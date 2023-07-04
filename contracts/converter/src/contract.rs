#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{CosmosMsg, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, WasmMsg, WasmQuery, QuerierWrapper, QueryRequest };
use osmosis_std::types::osmosis::tokenfactory::v1beta1::{ MsgCreateDenom, MsgMint, MsgBurn };
use osmosis_std::types::cosmos::base::v1beta1::Coin;
use osmosis_std::types::cosmos::bank::v1beta1::MsgSend;
use osmosis_std::types::cosmos::authz::v1beta1::MsgExec;
use osmosis_std::shim::Any;
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cw20::{Cw20ExecuteMsg, Cw20QueryMsg, TokenInfoResponse};
use crate::state::{Denom, DENOMSDATA};

const BANK_SEND_TYPE_URL: &str = "/cosmos.bank.v1beta1.MsgSend";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match _msg {
        ExecuteMsg::ConvertCw20ToCoin { token, amount, recipient } => conver_cw20_to_coin(_deps, _env, _info, token, amount, recipient),
        ExecuteMsg::ConvertCoinToCw20 { token, amount, recipient } => conver_coin_to_cw20(_deps, _env, _info, token, amount, recipient),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

#[cfg(test)]
mod tests {}

pub fn query_token_info(
    querier: &QuerierWrapper,
    contract_addr: Addr,
) -> StdResult<TokenInfoResponse> {
    let token_info: TokenInfoResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: contract_addr.to_string(),
        msg: to_binary(&Cw20QueryMsg::TokenInfo {})?,
    }))?;

    Ok(token_info)
}

fn conver_cw20_to_coin( deps: DepsMut, env: Env, info: MessageInfo, token: String, amount_to_mint: Uint128, recipient: String) -> Result<Response, ContractError> {
    let sender: String = env.contract.address.into();
    let mut messages = vec![];

    let token_info: TokenInfoResponse = query_token_info(&deps.querier, Addr::unchecked(token.clone()))?;
    let init_msg: CosmosMsg;

    let mut response = Response::new();

    let load_denoms_data = match DENOMSDATA.may_load(deps.storage, &String::from(token_info.name.clone()))? {
        None => {
            init_msg = (MsgCreateDenom {
                sender: sender.clone(),
                subdenom: String::from(token_info.name.clone()),
            })
            .into();

            let new_denoms_data = Denom {
                token: String::from(token.clone()),
                denom: String::from(token_info.name.clone()),
            };

            DENOMSDATA.save(deps.storage, &String::from(token_info.name.clone()), &new_denoms_data)?;
            response = response.add_message(init_msg);
            new_denoms_data
        }
        Some(load_denoms_data) => load_denoms_data,
    };

    assert_eq!(load_denoms_data.token.clone(), token.clone().to_string());

    let msg_mint: CosmosMsg = MsgMint { 
        sender: sender.clone(), 
        amount: Some(Coin {
            denom: format!("factory/{}/{}", sender.clone(), String::from(token_info.name.clone())),
            amount: String::from(amount_to_mint.clone()),
        }),
    }.into();

    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token.clone(),
        msg: to_binary(&Cw20ExecuteMsg::IncreaseAllowance {
            spender: sender.clone(),
            amount: amount_to_mint.clone(),
            expires: None,
        })?,
        funds: vec![],
    }));

    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token.clone(),
        msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
            owner: info.sender.to_string(),
            recipient: sender.clone(),
            amount: amount_to_mint.clone(),
        })?,
        funds: vec![],
    }));

    let send_msg: CosmosMsg = MsgSend {
        from_address: sender.clone(),
        to_address: String::from(recipient),
        amount: vec![Coin {
            denom: format!("factory/{}/{}", sender.clone(), String::from(token_info.name.clone())),
            amount: String::from(amount_to_mint.clone()),
        }],
    }.into();

    Ok(response
        .add_attribute("action", "convert_cw20_to_coin")
        .add_attribute("sender", &info.sender)
        .add_attribute("subdenom", String::from(token_info.name.clone()))
        .add_attribute("amount", String::from(amount_to_mint.clone()))
        .add_messages(messages)
        .add_message(msg_mint)
        .add_message(send_msg))
}

fn conver_coin_to_cw20( deps: DepsMut, env: Env, info: MessageInfo, token: String, amount_to_burn: Uint128, recipient: String ) -> Result<Response, ContractError> {
    let sender: String = env.contract.address.into();
    let mut messages = vec![];


    let token_info: TokenInfoResponse = query_token_info(&deps.querier, Addr::unchecked(token.clone()))?;

    let msg_send = MsgSend {
        from_address: String::from(recipient.clone()),
        to_address: sender.clone(),
        amount: vec![Coin {
            denom: format!("factory/{}/{}", sender.clone(), String::from(token_info.name.clone())),
            amount: String::from(amount_to_burn.clone()),
        }],
    };

    let msg_send_binary: cosmwasm_std::Binary = msg_send.into();


    let msg_send_any = Any {
        type_url: String::from(BANK_SEND_TYPE_URL),
        value: msg_send_binary.to_vec(),
    };

    let send_msg: CosmosMsg = MsgExec {
        grantee: String::from(sender.clone()),
        msgs: vec![msg_send_any],
    }.into();

    let msg_burn: CosmosMsg = MsgBurn { 
        sender: sender.clone(), 
        amount: Some(Coin {
            denom: format!("factory/{}/{}", sender.clone(), String::from(token_info.name.clone())),
            amount: String::from(amount_to_burn.clone()),
        }),
    }.into();

    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token.clone(),
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: String::from(recipient.clone()),
            amount: amount_to_burn.clone(),
        })?,
        funds: vec![],
    }));

    Ok(Response::new()
        .add_attribute("action", "conver_coin_to_cw20")
        .add_attribute("sender", &info.sender)
        .add_attribute("subdenom", String::from(token_info.name.clone()))
        .add_attribute("amount", String::from(amount_to_burn.clone()))
        .add_message(send_msg)
        .add_message(msg_burn)
        .add_messages(messages))
}