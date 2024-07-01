use cosmwasm_std::{
    Addr, Attribute, Binary, Coin, ContractResult, Empty, Querier, QuerierWrapper, SystemResult, to_binary,
    testing::{mock_env, mock_info, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR},
    Uint128, from_json, WasmQuery, QuerierResult, SystemError,
};
use cw721::{Cw721QueryMsg, OwnerOfResponse, Cw721ReceiveMsg};
use crate::{
    contract::{instantiate, execute, query},
    msg::{InstantiateMsg, ExecuteMsg, QueryMsg, Cw721HookMsg, OriginCw721Response, WrappedCw721Response, IsUnwrappableResponse},
    state::{OriginInfo, WrappedInfo, WRAPPED_INFO, ORIGIN_INFO, WRAPPED_TOKEN_ADDRESS},
    testing::mock_querier::{mock_dependencies_custom, MOCK_CW721_CONTRACT}
};
use andromeda_std::{
    common::{encode_binary, milliseconds::MillisecondsDuration},
    amp::{AndrAddr, Recipient},
    error::ContractError,
};
use andromeda_non_fungible_tokens::cw721::TokenExtension;
use andromeda_std::testing::mock_querier::{MOCK_APP_CONTRACT, MOCK_KERNEL_CONTRACT};

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies_custom(&[]);
    let msg = InstantiateMsg {
        kernel_address: MOCK_KERNEL_CONTRACT.to_string(),
        owner: Some("creator".to_owned()),
    };
    let info = mock_info(MOCK_CW721_CONTRACT, &[]);
    let env = mock_env();

    let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(0, res.messages.len());
}

#[test]
fn test_mint_wrapped_nft() {
    let mut deps = mock_dependencies_custom(&[]);
    let msg = InstantiateMsg {
        kernel_address: MOCK_KERNEL_CONTRACT.to_string(),
        owner: Some("creator".to_owned()),
    };
    let info = mock_info(MOCK_CW721_CONTRACT, &[]);
    let env = mock_env();

    instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    let hook_msg = Cw721HookMsg::MintWrappedNft {
        wrapped_token: AndrAddr::from_string(MOCK_CW721_CONTRACT.to_string()),
        wrapped_token_id: "wrapped_token".to_string(),
        wrapped_token_owner: "wrapped_owner".to_string(),
        wrapped_token_uri: None,
        wrapped_token_extension: TokenExtension { publisher: "publisher".to_string() },
        unwrappable: true,
    };

    let msg = ExecuteMsg::ReceiveNft(Cw721ReceiveMsg {
        sender: MOCK_CW721_CONTRACT.to_string(),
        token_id: "token1".to_string(),
        msg: encode_binary(&hook_msg).unwrap(),
    });

    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(res.attributes, vec![Attribute { key: "method".to_string(), value: "mint_wrapped_cw721".to_string() }]);

    // Query to verify the wrapped token info
    let query_msg = QueryMsg::WrappedCw721 {
        origin_token: AndrAddr::from_string(MOCK_CW721_CONTRACT.to_string()),
        origin_token_id: "token1".to_string(),
    };

    let res: WrappedCw721Response = from_json(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
    assert_eq!(res.wrapped_token.to_string(), MOCK_CW721_CONTRACT.to_string());
    assert_eq!(res.wrapped_token_id, "wrapped_token".to_string());
}

#[test]
fn test_unwrap_nft() {
    let mut deps = mock_dependencies_custom(&[]);
    let msg = InstantiateMsg {
        kernel_address: MOCK_KERNEL_CONTRACT.to_string(),
        owner: Some("creator".to_owned()),
    };
    let info = mock_info(MOCK_CW721_CONTRACT, &[]);
    let env = mock_env();

    instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    let hook_msg = Cw721HookMsg::MintWrappedNft {
        wrapped_token: AndrAddr::from_string(MOCK_CW721_CONTRACT.to_string()),
        wrapped_token_id: "wrapped_token".to_string(),
        wrapped_token_owner: "wrapped_owner".to_string(),
        wrapped_token_uri: None,
        wrapped_token_extension: TokenExtension { publisher: "publisher".to_string() },
        unwrappable: true,
    };

    let msg = ExecuteMsg::ReceiveNft(Cw721ReceiveMsg {
        sender: MOCK_CW721_CONTRACT.to_string(),
        token_id: "token1".to_string(),
        msg: encode_binary(&hook_msg).unwrap(),
    });

    execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Query to verify the original token info
    let query_msg = QueryMsg::OriginCw721 {
        wrapped_token: AndrAddr::from_string(MOCK_CW721_CONTRACT.to_string()),
        wrapped_token_id: "wrapped_token".to_string(),
    };

    let res: OriginCw721Response = from_json(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
    assert_eq!(res.origin_token.to_string(), MOCK_CW721_CONTRACT.to_string());
    assert_eq!(res.origin_token_id, "token1".to_string());

    let unwrap_msg = ExecuteMsg::ReceiveNft(Cw721ReceiveMsg {
        sender: MOCK_CW721_CONTRACT.to_string(),
        token_id: "wrapped_token".to_string(),
        msg: encode_binary(&Cw721HookMsg::UnwrapNft {
            recipient: AndrAddr::from_string("recipient".to_string()),
            wrapped_token: AndrAddr::from_string(MOCK_CW721_CONTRACT.to_string()),
            wrapped_token_id: "wrapped_token".to_string(),
        }).unwrap(),
    });

    let res = execute(deps.as_mut(), env.clone(), info.clone(), unwrap_msg).unwrap();
    assert_eq!(res.attributes, vec![Attribute { key: "method".to_string(), value: "unwrap_cw721".to_string() }]);
}

#[test]
fn test_is_unwrappable() {
    let mut deps = mock_dependencies_custom(&[]);
    let msg = InstantiateMsg {
        kernel_address: MOCK_KERNEL_CONTRACT.to_string(),
        owner: Some("creator".to_owned()),
    };
    let info = mock_info(MOCK_CW721_CONTRACT, &[]);
    let env = mock_env();

    instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    let hook_msg = Cw721HookMsg::MintWrappedNft {
        wrapped_token: AndrAddr::from_string(MOCK_CW721_CONTRACT.to_string()),
        wrapped_token_id: "wrapped_token".to_string(),
        wrapped_token_owner: "wrapped_owner".to_string(),
        wrapped_token_uri: None,
        wrapped_token_extension: TokenExtension { publisher: "publisher".to_string() },
        unwrappable: true,
    };

    let msg = ExecuteMsg::ReceiveNft(Cw721ReceiveMsg {
        sender: MOCK_CW721_CONTRACT.to_string(),
        token_id: "token1".to_string(),
        msg: encode_binary(&hook_msg).unwrap(),
    });

    execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    let query_msg = QueryMsg::IsUnwrappable {
        wrapped_token: AndrAddr::from_string(MOCK_CW721_CONTRACT.to_string()),
        wrapped_token_id: "wrapped_token".to_string(),
    };

    let res: IsUnwrappableResponse = from_json(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
    assert!(res.is_unwrappable);
}

#[test]
fn test_query_wrapped_token() {
    let mut deps = mock_dependencies_custom(&[]);
    let msg = InstantiateMsg {
        kernel_address: MOCK_KERNEL_CONTRACT.to_string(),
        owner: Some("creator".to_owned()),
    };
    let info = mock_info(MOCK_CW721_CONTRACT, &[]);
    let env = mock_env();

    instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    let hook_msg = Cw721HookMsg::MintWrappedNft {
        wrapped_token: AndrAddr::from_string(MOCK_CW721_CONTRACT.to_string()),
        wrapped_token_id: "wrapped_token".to_string(),
        wrapped_token_owner: "wrapped_owner".to_string(),
        wrapped_token_uri: None,
        wrapped_token_extension: TokenExtension { publisher: "publisher".to_string() },
        unwrappable: true,
    };

    let msg = ExecuteMsg::ReceiveNft(Cw721ReceiveMsg {
        sender: MOCK_CW721_CONTRACT.to_string(),
        token_id: "token1".to_string(),
        msg: encode_binary(&hook_msg).unwrap(),
    });

    execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    let query_msg = QueryMsg::WrappedCw721 {
        origin_token: AndrAddr::from_string(MOCK_CW721_CONTRACT.to_string()),
        origin_token_id: "token1".to_string(),
    };

    let res: WrappedCw721Response = from_json(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
    assert_eq!(res.wrapped_token.to_string(), MOCK_CW721_CONTRACT.to_string());
    assert_eq!(res.wrapped_token_id, "wrapped_token".to_string());
}
