#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, ensure, Response, CosmosMsg, WasmMsg, from_json, attr, Empty};
use andromeda_std::{
    ado_base::{InstantiateMsg as BaseInstantiateMsg, permissioning::Permission},
    ado_contract::ADOContract,
    common::{
        encode_binary, milliseconds::MillisecondsDuration, milliseconds::Milliseconds, context::ExecuteContext,
    },
    error::ContractError,
    amp::{AndrAddr, Recipient},
};
use andromeda_non_fungible_tokens::cw721::ExecuteMsg as Cw721ExecuteMsg;

use crate::msg::{ ExecuteMsg, InstantiateMsg, Cw721HookMsg, UnlockTimeResponse, NftDetailsResponse, IsLockedResponse, QueryMsg };
use crate::state::{TIMELOCKS, TimelockInfo};

use cw721::Cw721ReceiveMsg;
use cw2::set_contract_version;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:andromeda-cw721-timelock";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const SEND_NFT_ACTION: &str = "SEND_NFT";

const ONE_DAY: u64 = 24 * 60 * 60;
const ONE_YEAR: u64 = 365 * 24 * 60 * 60;

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

pub fn handle_execute(ctx: ExecuteContext, msg: ExecuteMsg) -> Result<Response, ContractError> {

    match msg {
        ExecuteMsg::ReceiveNft(msg) => handle_receive_cw721(ctx, msg),
        ExecuteMsg::ClaimNft {
            cw721_contract,
            token_id,
        } => execute_claim_cw721(ctx, cw721_contract, token_id),
        _ => ADOContract::default().execute(ctx, msg),
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
        Cw721HookMsg::TimelockNft {
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
    _sender: String,
    token_id: String,
    lock_duration: MillisecondsDuration,
    recipient: Recipient,
) -> Result<Response<Empty>, ContractError> {
    let ExecuteContext {
        deps,
        info,
        env,
        ..
    } = ctx;

    ensure!(
        lock_duration.seconds() >= ONE_DAY,
        ContractError::LockTimeTooShort {}
    );
    ensure!(
        lock_duration.seconds() <= ONE_YEAR,
        ContractError::LockTimeTooLong {}
    );

    let nft_address = info.sender.to_string();
    let lock_id = format!("{}:{}", nft_address, token_id);

    let recipient_addr = AndrAddr::from_string(recipient.get_addr()).get_raw_address(&deps.as_ref())?;
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

fn execute_claim_cw721(
    ctx: ExecuteContext,
    cw721_contract: AndrAddr,
    token_id: String,
) -> Result<Response<Empty>, ContractError> {
    let ExecuteContext {
        deps,
        env,
        ..
    } = ctx;

    let lock_id = format!("{}:{}", cw721_contract, token_id);
    let timelock_info: Option<TimelockInfo> = Some(TIMELOCKS.load(deps.storage, lock_id.as_str())?);

    if let Some(timelock_info) = timelock_info {
        if env.block.time.seconds() < timelock_info.unlock_time.seconds() {
            return Err(ContractError::LockedNFT {});
        }

        let transfer_msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: cw721_contract.into_string().clone(),
            msg: encode_binary(&Cw721ExecuteMsg::TransferNft {
                recipient: AndrAddr::from_string(timelock_info.recipient.to_string()),
                // recipient: timelock_info.recipient.to_string(),
                token_id: token_id.clone(),
            })?,
            funds: vec![],
        });

        TIMELOCKS.remove(deps.storage, lock_id.as_str());

        Ok(Response::new()
            .add_message(transfer_msg)
            .add_attribute("method", "claim_nft")
        )
    } else {
        // Return a custom error if the timelock_info does not exist
        Err(ContractError::NFTNotFound {})
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
    deps: Deps,
    env: Env,
    msg: QueryMsg,
) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::UnlockTime { cw721_contract, token_id } => encode_binary(&query_unlock_time(deps, cw721_contract, token_id)?),
        QueryMsg::NftDetails { cw721_contract, token_id } => encode_binary(&query_nft_details(deps, cw721_contract, token_id)?),
        QueryMsg::IsLocked { cw721_contract, token_id } => encode_binary(&query_is_locked(deps, env, cw721_contract, token_id)?),
        _ => ADOContract::default().query(deps, env, msg),
    }
}

fn query_unlock_time(
    deps: Deps,
    cw721_contract: AndrAddr,
    token_id: String,
) -> Result<UnlockTimeResponse, ContractError> {
    let lock_id = format!("{}:{}", cw721_contract.into_string(), token_id);
    let timelock = TIMELOCKS.load(deps.storage, lock_id.as_str())?;
    Ok(UnlockTimeResponse {
        unlock_time: timelock.unlock_time.seconds(),
    })
}

fn query_nft_details(
    deps: Deps,
    cw721_contract: AndrAddr,
    token_id: String,
) -> Result<NftDetailsResponse, ContractError> {
    let lock_id = format!("{}:{}", cw721_contract.into_string(), token_id);
    let timelock = TIMELOCKS.load(deps.storage, lock_id.as_str())?;
    Ok(NftDetailsResponse {
        unlock_time: timelock.unlock_time.seconds(),
        recipient: timelock.recipient,
    })
}

fn query_is_locked(
    deps: Deps,
    env: Env,
    cw721_contract: AndrAddr,
    token_id: String,
) -> Result<IsLockedResponse, ContractError> {
    let lock_id = format!("{}:{}", cw721_contract.into_string(), token_id);
    let timelock = TIMELOCKS.load(deps.storage, lock_id.as_str())?;
    let unlock_time = timelock.unlock_time.seconds();
    let current_time = env.block.time.seconds();
    let is_locked = unlock_time > current_time;

    Ok(IsLockedResponse {
        is_locked,
    })
}
