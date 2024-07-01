#[cfg(not(feature = "imported"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, ensure, from_json, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, WasmMsg
};

// use crate::state::{
//     is_archived, ANDR_MINTER, ARCHIVED, BATCH_MINT_ACTION, MINT_ACTION, TRANSFER_AGREEMENTS,
// };
use andromeda_non_fungible_tokens::cw721::{
    ExecuteMsg as AndrCw721ExecuteMsg, TokenExtension,
};
use crate::{
    msg::{Cw721HookMsg, ExecuteMsg, InstantiateMsg, QueryMsg, OriginCw721Response, WrappedCw721Response, IsUnwrappableResponse},
    state::{WrappedInfo, WRAPPED_INFO, OriginInfo, ORIGIN_INFO, WRAPPED_TOKEN_ADDRESS},
};

use andromeda_std::common::encode_binary;
use andromeda_std::{
    // ado_base::{AndromedaMsg, AndromedaQuery},
    ado_contract::ADOContract,
    amp::AndrAddr,
    common::context::ExecuteContext,
};

use andromeda_std::{
    ado_base::InstantiateMsg as BaseInstantiateMsg,
    // common::Funds,
    error::ContractError,
};
use cw721::Cw721ReceiveMsg;

const CONTRACT_NAME: &str = "crates.io:andromeda-wrapped-cw721";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "imported"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {

    let contract = ADOContract::default();

    let resp = contract.instantiate(
        deps.storage,
        env,
        deps.api,
        &deps.querier,
        info.clone(),
        BaseInstantiateMsg {
            ado_type: CONTRACT_NAME.to_string(),
            ado_version: CONTRACT_VERSION.to_string(),
            kernel_address: msg.kernel_address,
            owner: msg.owner,
        },
    )?;

    Ok(
        resp
        .add_attribute("method", "instantiate")
    )
}

#[cfg_attr(not(feature = "imported"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    if let ExecuteMsg::AMPReceive(pkt) = msg {
        ADOContract::default().execute_amp_receive(
            ExecuteContext::new(deps, info, env),
            pkt,
            handle_execute,
        )
    } else {
        let ctx = ExecuteContext::new(deps, info, env);
        handle_execute(ctx, msg)
    }
}

fn handle_execute(ctx: ExecuteContext, msg: ExecuteMsg) -> Result<Response, ContractError> {

    match msg {
        // ExecuteMsg::SetWrappedTokenAddress { token_address } => execute_set_wrapped_token_address(ctx, token_address),
        ExecuteMsg::ReceiveNft(msg) => handle_receive_cw721(ctx, msg),
        _ => ADOContract::default().execute(ctx, msg),
    }
}

fn handle_receive_cw721(ctx: ExecuteContext, msg: Cw721ReceiveMsg) -> Result<Response, ContractError> {

    // ADOContract::default().is_permissioned(
    //     ctx.deps.storage,
    //     ctx.env.clone(),
    //     SEND_NFT_ACTION,
    //     ctx.info.sender.clone(),
    // )?;

    match from_json(&msg.msg)? {
        Cw721HookMsg::MintWrappedNft { 
            wrapped_token,
            wrapped_token_id,
            wrapped_token_owner,
            wrapped_token_uri,
            wrapped_token_extension,
            unwrappable,
        } => execute_mint_wrapped_cw721(
            ctx,
            wrapped_token,
            wrapped_token_id,
            wrapped_token_owner,
            wrapped_token_uri,
            wrapped_token_extension,
            unwrappable,
            // msg.sender,
            msg.token_id,
        ),
        Cw721HookMsg::UnwrapNft { 
            recipient,
            wrapped_token,
            wrapped_token_id 
        } => execute_unwrap_cw721(
            ctx,
            recipient,
            wrapped_token,
            wrapped_token_id,
        ),
    }
}

fn execute_mint_wrapped_cw721(
    ctx: ExecuteContext,
    wrapped_token: AndrAddr,
    wrapped_token_id: String,
    wrapped_token_owner: String,
    wrapped_token_uri: Option<String>,
    wrapped_token_extension: TokenExtension,
    unwrappable: bool,
    // origin_token: String,
    origin_token_id: String,
) -> Result<Response, ContractError> {

    let ExecuteContext {
        deps,
        info,
        // env,
        ..
    } = ctx;

    // Todo: confirm that wrapped_token of parameter is same as address from storage!

    let cw721_address = info.sender.to_string();
    let wrapped_id = format!("{}:{}", wrapped_token.to_string(), wrapped_token_id);
    let origin_id = format!("{}:{}", cw721_address, origin_token_id);

    let origin_info = OriginInfo {
        origin_token: AndrAddr::from_string(cw721_address),
        origin_token_id: origin_token_id.clone(),
        unwrappable: unwrappable.clone(),
    };
    let wrapped_info = WrappedInfo {
        wrapped_token: wrapped_token.clone(),
        wrapped_token_id: wrapped_token_id.clone(),
        unwrappable: unwrappable.clone(),
    };

    ORIGIN_INFO.save(deps.storage, wrapped_id.as_str(), &origin_info)?;
    WRAPPED_INFO.save(deps.storage, origin_id.as_str(), &wrapped_info)?;

    let mint_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: wrapped_token.into_string().clone(),
        msg: encode_binary(&AndrCw721ExecuteMsg::Mint { 
            token_id: wrapped_token_id.clone(), 
            owner: wrapped_token_owner.clone(), 
            token_uri: wrapped_token_uri.clone(), 
            extension: wrapped_token_extension.clone(),
        })?,
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(mint_msg)
        .add_attribute("method", "mint_wrapped_cw721")
    )
}

fn execute_unwrap_cw721(
    ctx: ExecuteContext,
    recipient: AndrAddr,
    wrapped_token: AndrAddr,
    wrapped_token_id: String,
) -> Result<Response, ContractError> {

    let ExecuteContext {
        deps,
        // info,
        // env,
        ..
    } = ctx;


    let wrapped_id = format!("{}:{}", wrapped_token.to_string(), wrapped_token_id);
    let origin_info: Option<OriginInfo> = Some(ORIGIN_INFO.load(deps.storage, wrapped_id.as_str())?);

    if let Some(origin_info) = origin_info {
        ensure!(
            origin_info.unwrappable,
            ContractError::UnwrappingDisabled {}
        );

        let origin_token = origin_info.origin_token;
        let origin_token_id = origin_info.origin_token_id;
        let origin_id = format!("{}:{}", origin_token.to_string(), wrapped_token_id);

        let unwrap_cw721_msg = CosmosMsg::Wasm(WasmMsg::Execute { 
            contract_addr: origin_token.into_string().clone(), 
            msg: encode_binary(&AndrCw721ExecuteMsg::TransferNft { 
                recipient: AndrAddr::from_string(recipient.to_string()), 
                token_id: origin_token_id.clone(), 
            })?, 
            funds: vec![], 
        });

        let burn_wrapped_cw721_msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: wrapped_token.into_string().clone(),
            msg: encode_binary(&AndrCw721ExecuteMsg::Burn { token_id: wrapped_token_id.clone() })?,
            funds: vec![],
        });

        WRAPPED_INFO.remove(deps.storage, origin_id.as_str());
        ORIGIN_INFO.remove(deps.storage, wrapped_id.as_str());

        Ok(
            Response::new()
            .add_message(burn_wrapped_cw721_msg)
            .add_message(unwrap_cw721_msg)
            .add_attribute("method", "unwrap_cw721")
        )
    } else {
        return Err(ContractError::NFTNotFound {})
    }
}

#[cfg_attr(not(feature = "imported"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg{
        QueryMsg::OriginCw721 { wrapped_token, wrapped_token_id } => {
            Ok(encode_binary(&query_origin_cw721(deps, wrapped_token, wrapped_token_id)?)?)
        },
        QueryMsg::WrappedCw721 { origin_token, origin_token_id } => {
            Ok(encode_binary(&query_wrapped_cw721(deps, origin_token, origin_token_id)?)?)
        },
        QueryMsg::IsUnwrappable { wrapped_token, wrapped_token_id } => {
            Ok(encode_binary(&query_is_unwrappable(deps, wrapped_token, wrapped_token_id)?)?)
        },
        _ => ADOContract::default().query(deps, env, msg),
    }
}

pub fn query_origin_cw721(deps: Deps, wrapped_token: AndrAddr, wrapped_token_id: String) -> Result<OriginCw721Response, ContractError> {
    let wrapped_id = format!("{}:{}", wrapped_token.get_raw_address(&deps)?, wrapped_token_id);
    let origin_info: Option<OriginInfo> = Some(ORIGIN_INFO.load(deps.storage, wrapped_id.as_str())?);

    if let Some(origin_info) = origin_info {
        Ok(OriginCw721Response {
            origin_token: origin_info.origin_token,
            origin_token_id: origin_info.origin_token_id,
        })
    } else {
        return Err(ContractError::TokenNotWrappedByThisContract {})
    }
}

pub fn query_wrapped_cw721(deps: Deps, origin_token: AndrAddr, origin_token_id: String) -> Result<WrappedCw721Response, ContractError> {
    let origin_id = format!("{}:{}", origin_token.get_raw_address(&deps)?, origin_token_id);
    let wrapped_info: Option<WrappedInfo> = Some(WRAPPED_INFO.load(deps.storage, origin_id.as_str())?);

    if let Some(wrapped_info) = wrapped_info {
        Ok(WrappedCw721Response {
            wrapped_token: wrapped_info.wrapped_token,
            wrapped_token_id: wrapped_info.wrapped_token_id,
        })
    } else {
        return Err(ContractError::TokenNotWrappedByThisContract {})
    }
}

pub fn query_is_unwrappable(deps: Deps, wrapped_token: AndrAddr, wrapped_token_id: String) -> Result<IsUnwrappableResponse, ContractError> {
    let wrapped_id = format!("{}:{}", wrapped_token.get_raw_address(&deps)?, wrapped_token_id);
    let origin_info: Option<OriginInfo> = Some(ORIGIN_INFO.load(deps.storage, wrapped_id.as_str())?);

    if let Some(origin_info) = origin_info {
        Ok(IsUnwrappableResponse {
            is_unwrappable: origin_info.unwrappable,
        })
    } else {
        return Err(ContractError::TokenNotWrappedByThisContract {})
    }
}
