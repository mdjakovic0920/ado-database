#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, ensure, Response, CosmosMsg, WasmMsg, from_json, attr};
use andromeda_std::{
    ado_base::{InstantiateMsg as BaseInstantiateMsg, permissioning::Permission},
    ado_contract::{
        ADOContract,
    },
    common::{
        encode_binary, milliseconds::MillisecondsDuration, milliseconds::Milliseconds, context::ExecuteContext,
        // actions::call_action::get_action_name
    },
    error::ContractError,
    amp::{AndrAddr, Recipient},
};
use andromeda_non_fungible_tokens::cw721::ExecuteMsg as Cw721ExecuteMsg;

// use crate::msg::{ ExecuteMsg, InstantiateMsg, QueryMsg, UnlockTimeResponse, NftDetailsResponse };
use crate::msg::{ ExecuteMsg, InstantiateMsg, Cw721HookMsg };

use crate::state::{TIMELOCKS, TimelockInfo};

use cw721::{Cw721QueryMsg, Cw721ReceiveMsg, OwnerOfResponse};
use cw2::set_contract_version;
use cw_utils::nonpayable;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:andromeda-cw721-timelock";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const SEND_NFT_ACTION: &str = "SEND_NFT";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
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

    if let Some(authorized_token_addresses) = msg.authorized_token_addresses {
        if !authorized_token_addresses.is_empty() {
            ADOContract::default().permission_action(SEND_NFT_ACTION, deps.storage)?;
        }

        for token_address in authorized_token_addresses {
            let addr = token_address.get_raw_address(&deps.as_ref())?;
            ADOContract::set_permission(
                deps.storage,
                SEND_NFT_ACTION,
                addr,
                Permission::Whitelisted(None),
            )?;
        }
    }

    Ok(
        resp
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let ctx = ExecuteContext::new(deps, info, env);

    match msg {
        ExecuteMsg::AMPReceive(pkt) => {
            ADOContract::default().execute_amp_receive(ctx, pkt, handle_execute)
        }
        _ => handle_execute(ctx, msg),
    }
}

pub fn handle_execute(mut ctx: ExecuteContext, msg: ExecuteMsg) -> Result<Response, ContractError> {
    // let action = get_action_name(CONTRACT_NAME, msg.as_ref());

    match msg {
        ExecuteMsg::ReceiveNft(msg) => handle_receive_cw721(ctx, msg),
        _ => ADOContract::default().execute(ctx, msg,)
    }
}

fn handle_receive_cw721(ctx: ExecuteContext, msg: Cw721ReceiveMsg) -> Result<Response, ContractError> {

    ADOContract::default().is_permissioned(
        ctx.deps.storage,
        ctx.env.clone(),
        SEND_NFT_ACTION,
        ctx.info.sender.clone(),
    )?;

    match from_json(&msg.msg)? {
        Cw721HookMsg::TimelockNFT {
            // cw721_contract,
            // token_id,
            lock_duration,
            recipient,
        } => execute_timelock_cw721(
            ctx,
            msg.sender,
            msg.token_id,
            lock_duration,
            recipient,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn execute_timelock_cw721(
    ctx: ExecuteContext,
    sender: String,
    token_id: String,
    lock_duration: MillisecondsDuration,
    recipient: Recipient,
) -> Result<Response, ContractError> {
    let ExecuteContext {
        mut deps,
        info,
        env,
        ..
    } = ctx;

    ensure!(
        !lock_duration.seconds() > 24 * 60 * 60,
        ContractError::LockTimeTooShort {}
    );
    ensure!(
        !lock_duration.seconds() < 365* 24 * 60 * 60,
        ContractError::LockTimeTooLong {}
    );

    let nft_address = info.sender.to_string();
    let lock_id = format!("{}:{}", nft_address, token_id);

    let recipient_addr = deps.api.addr_validate(&recipient.get_addr())?;
    let timelock_info = TimelockInfo {
        unlock_time: Milliseconds::from_seconds(env.block.time.seconds() + lock_duration.seconds()),
        recipient: recipient_addr,
    };

    TIMELOCKS.save(deps.storage, lock_id.as_str(), &timelock_info)?;

    Ok(Response::new()
        .add_attributes(vec![
            attr("method", "timelock_cw721"),
            attr("lock_id", lock_id),
        ])
    )
}

// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn execute(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     msg: ExecuteMsg,
// ) -> Result<Response, ContractError> {
//     match msg {
//         ExecuteMsg::SetTimelock { cw721_contract, token_id, unlock_time, recipient } => {
//             try_set_timelock(deps, env, info, cw721_contract, token_id, unlock_time, recipient)
//         },
//         ExecuteMsg::Claim { cw721_contract, token_id } => try_claim_nft(deps, env, cw721_contract, token_id),
//         _ => todo!(),
//     }
// }

// // Before call try_set_timelock, the owner of cw721 approve cw721-timelock as a spender.
// pub fn try_set_timelock(
//     deps: DepsMut,
//     env: Env,
//     _info: MessageInfo,
//     cw721_contract: String,
//     token_id: String,
//     unlock_time: u64,
//     recipient: String,
// ) -> Result<Response, ContractError> {

//     // let contract = ADOContract::default();
//     // ensure!(
//     //     contract.is_contract_owner(deps.storage, _info.sender.as_str())?,
//     //     ContractError::Unauthorized {}
//     // );
//     // println!("Sender: {}", _info.sender.as_str());
//     // let state = CONFIG.load(deps.storage)?;
//     // if state.owner != info.sender {
//     //     return Err(StdError::generic_err("Unauthorized"));
//     // }

//     let recipient_addr = deps.api.addr_validate(&recipient)?;
//     let timelock_info = TimelockInfo {
//         unlock_time: env.block.time.seconds() + unlock_time,
//         recipient: recipient_addr,
//     };

//     let lock_id = format!("{}:{}", cw721_contract, token_id);
//     TIMELOCKS.save(deps.storage, lock_id.as_str(), &timelock_info)?;

//     let send_msg = CosmosMsg::Wasm(WasmMsg::Execute {
//         contract_addr: cw721_contract.clone(),
//         msg: encode_binary(&Cw721ExecuteMsg::SendNft {
//             contract: AndrAddr::from_string(env.contract.address.to_string()),
//             token_id: token_id.clone(),
//             msg: Binary::from_base64("")?,
//         })?,
//         funds: vec![],
//     });

//     Ok(
//         Response::new()
//         .add_message(send_msg)
//         .add_attribute("method", "set_timelock")
//     )
// }

// pub fn try_claim_nft(
//     deps: DepsMut,
//     env: Env,
//     cw721_contract: String,
//     token_id: String,
// ) -> Result<Response, ContractError> {
//     let lock_id = format!("{}:{}", cw721_contract, token_id);
//     let timelock_info: Option<TimelockInfo> = Some(TIMELOCKS.load(deps.storage, lock_id.as_str())?);

//     if let Some(timelock_info) = timelock_info {
//         if env.block.time.seconds() < timelock_info.unlock_time {
//             return Err(ContractError::LockedNFT {});
//         }

//         let transfer_msg = CosmosMsg::Wasm(WasmMsg::Execute {
//             contract_addr: cw721_contract.clone(),
//             msg: encode_binary(&Cw721ExecuteMsg::TransferNft {
//                 recipient: AndrAddr::from_string(timelock_info.recipient.to_string()),
//                 // recipient: timelock_info.recipient.to_string(),
//                 token_id: token_id.clone(),
//             })?,
//             funds: vec![],
//         });

//         TIMELOCKS.remove(deps.storage, lock_id.as_str());

//         Ok(Response::new()
//             .add_message(transfer_msg)
//             .add_attribute("method", "claim_nft")
//         )
//     } else {
//         // Return a custom error if the timelock_info does not exist
//         Err(ContractError::NFTNotFound {})
//     }
// }

// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn query(
//     deps: Deps,
//     _env: Env,
//     msg: QueryMsg,
// ) -> Result<Binary, ContractError> {
//     match msg {
//         QueryMsg::UnlockTime { cw721_contract, token_id } => encode_binary(&query_unlock_time(deps, cw721_contract, token_id)?),
//         QueryMsg::NftDetails { cw721_contract, token_id } => encode_binary(&query_nft_details(deps, cw721_contract, token_id)?),
//         _ => todo!(),
//     }
// }

// fn query_unlock_time(
//     deps: Deps,
//     cw721_contract: String,
//     token_id: String,
// ) -> Result<UnlockTimeResponse, ContractError> {
//     let lock_id = format!("{}:{}", cw721_contract, token_id);
//     let timelock = TIMELOCKS.load(deps.storage, lock_id.as_str())?;
//     Ok(UnlockTimeResponse {
//         unlock_time: timelock.unlock_time,
//     })
// }

// fn query_nft_details(
//     deps: Deps,
//     cw721_contract: String,
//     token_id: String,
// ) -> Result<NftDetailsResponse, ContractError> {
//     let lock_id = format!("{}:{}", cw721_contract, token_id);
//     let timelock = TIMELOCKS.load(deps.storage, lock_id.as_str())?;
//     Ok(NftDetailsResponse {
//         unlock_time: timelock.unlock_time,
//         recipient: timelock.recipient,
//     })
// }
