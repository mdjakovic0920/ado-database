use crate::contract::{execute, instantiate, query};
use crate::msg::{ExecuteMsg, InstantiateMsg, Cw721HookMsg, QueryMsg, OriginCw721Response, WrappedCw721Response, IsUnwrappableResponse};
use andromeda_std::testing::mock_querier::MOCK_KERNEL_CONTRACT;
use andromeda_std::ado_base::ownership::ContractOwnerResponse;
use cosmwasm_std::{Addr, Empty, testing::mock_env};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};
// use anyhow::Error;
use cw721_base::entry::{execute as cw721_execute, instantiate as cw721_instantiate, query as cw721_query};
use cw721::OwnerOfResponse;
use cw721_base::MinterResponse;
use andromeda_non_fungible_tokens::cw721::TokenExtension;
use cw721::Cw721ReceiveMsg;


use andromeda_std::{
    common::{milliseconds::MillisecondsDuration, encode_binary},
    amp::{AndrAddr, Recipient},
};

fn mock_app() -> App {
    App::default()
}

pub fn contract_wrapped_cw721() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(execute, instantiate, query);
    Box::new(contract)
}

pub fn contract_cw721() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(cw721_execute, cw721_instantiate, cw721_query);
    Box::new(contract)
}

#[test]
fn wrapped_cw721_test() {
    let mut router: App = mock_app();

    let addr1: Addr = Addr::unchecked("addr1");
    let addr2: Addr = Addr::unchecked("addr2");
    let origin_cw721_owner = Addr::unchecked("origin_cw721_owner");
    let recipient = Addr::unchecked("recipient");

    let wrapped_cw721_id: u64 = router.store_code(contract_wrapped_cw721());
    let cw721_id: u64 = router.store_code(contract_cw721());

    let wrapped_cw721_instantiate_msg: InstantiateMsg = InstantiateMsg {
        owner: None,
        kernel_address: MOCK_KERNEL_CONTRACT.to_string(),
    };

    let cw721_instantiate_msg: cw721_base::InstantiateMsg = cw721_base::msg::InstantiateMsg {
        name: "Test Wrapped CW721".to_string(),
        symbol: "W-CW721".to_string(),
        minter: addr2.to_string(),
    };

    let wrapped_cw721_addr: Addr = router.instantiate_contract(
        wrapped_cw721_id,
        addr1.clone(),
        &wrapped_cw721_instantiate_msg,
        &[],
        "Wrapped CW721",
        None,
    ).unwrap();

    let query_owner_msg = QueryMsg::Owner { };
    let owner_res: ContractOwnerResponse = router.wrap().query_wasm_smart(&wrapped_cw721_addr, &query_owner_msg).unwrap();
    
    // Assert that the owner of the Wrapped CW721 contract is addr1
    assert_eq!(addr1, owner_res.owner);

    let cw721_addr: Addr = router.instantiate_contract(
        cw721_id,
        addr2.clone(),
        &cw721_instantiate_msg,
        &[],
        "CW721",
        Some(origin_cw721_owner.to_string()),
    ).unwrap();

    let query_minter_msg: cw721_base::msg::QueryMsg<String> = cw721_base::msg::QueryMsg::Minter {};
    let minter_res: MinterResponse = router.wrap().query_wasm_smart(&cw721_addr, &query_minter_msg).unwrap();
    
    // Assert that the minter of the CW721 contract is addr2
    assert_eq!(Some(addr2.to_string()), minter_res.minter);

    let cw721_mint_msg: cw721_base::msg::ExecuteMsg<Empty, Empty> = cw721_base::msg::ExecuteMsg::Mint { 
        token_id: "token1".to_string(), 
        owner: origin_cw721_owner.to_string(), 
        token_uri: None, 
        extension: Empty::default(),
    };
    // Only minter of cw721 can execute mint function
    router.execute_contract(addr2.clone(), cw721_addr.clone(), &cw721_mint_msg, &[]).unwrap();

    let hook_msg = Cw721HookMsg::MintWrappedNft {
        wrapped_token: AndrAddr::from_string(cw721_addr.to_string()),
        wrapped_token_id: "wrapped_token".to_string(),
        wrapped_token_owner: origin_cw721_owner.to_string(),
        wrapped_token_uri: None,
        wrapped_token_extension: TokenExtension { publisher: "publisher".to_string() },
        unwrappable: true,
    };

    let send_cw721_msg: cw721_base::msg::ExecuteMsg<Empty, Empty> = cw721_base::msg::ExecuteMsg::SendNft { 
        contract: wrapped_cw721_addr.to_string(), 
        token_id: "token1".to_string(), 
        msg: encode_binary(&hook_msg).unwrap(),
    };

    let env = mock_env();
    router.execute_contract(origin_cw721_owner.clone(), cw721_addr.clone(), &send_cw721_msg, &[]).unwrap();

    let owner_res: OwnerOfResponse = router.wrap().query_wasm_smart(&cw721_addr, &cw721_base::msg::QueryMsg::OwnerOf { token_id: "token1".to_string(), include_expired: None }).unwrap();

    // Assert that the owner of token1 is now the Wrapped CW721 contract
    assert_eq!(wrapped_cw721_addr.to_string(), owner_res.owner);
    
    let query_origin_info_msg = QueryMsg::OriginCw721 { 
        wrapped_token: AndrAddr::from_string(cw721_addr.to_string()), 
        wrapped_token_id: "wrapped_token".to_string(), 
    };
    let origin_info_res: OriginCw721Response = router.wrap().query_wasm_smart(&wrapped_cw721_addr, &query_origin_info_msg).unwrap();
    
    // Assert that the origin token info is as expected
    assert_eq!(origin_info_res.origin_token.to_string(), cw721_addr.to_string());
    assert_eq!(origin_info_res.origin_token_id, "token1");

    let unwrap_msg = ExecuteMsg::ReceiveNft(Cw721ReceiveMsg {
        sender: wrapped_cw721_addr.to_string(),
        token_id: "wrapped_token".to_string(),
        msg: encode_binary(&Cw721HookMsg::UnwrapNft {
            recipient: AndrAddr::from_string(recipient.to_string()),
            wrapped_token: AndrAddr::from_string(cw721_addr.to_string()),
            wrapped_token_id: "wrapped_token".to_string(),
        }).unwrap(),
    });

    router.execute_contract(origin_cw721_owner.clone(), wrapped_cw721_addr.clone(), &unwrap_msg, &[]).unwrap();

    let owner_res: OwnerOfResponse = router.wrap().query_wasm_smart(&cw721_addr, &cw721_base::msg::QueryMsg::OwnerOf { token_id: "token1".to_string(), include_expired: None }).unwrap();

    // Assert that the owner of token1 is now the recipient
    assert_eq!(recipient.to_string(), owner_res.owner);
}