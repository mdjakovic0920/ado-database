// use cosmwasm_std::{
//     Addr, QueryRequest,
//     testing::{mock_env, mock_info, mock_dependencies},
//     WasmQuery, to_json_binary, from_json, Querier, Empty
// };
// use cw721::{Cw721QueryMsg, OwnerOfResponse};
// use crate::{
//     contract::{instantiate, execute, query},
//     msg::{InstantiateMsg, ExecuteMsg, QueryMsg, UnlockTimeResponse, NftDetailsResponse, TokenExtension},
//     testing::mock_querier::{MOCK_CW721_CONTRACT, MOCK_TOKEN_OWNER, MOCK_UNCLAIMED_TOKEN, mock_dependencies_custom}
// };
// use andromeda_std::{common::encode_binary, testing::mock_querier::MOCK_KERNEL_CONTRACT, error::ContractError};
// use cw721_base::{state::TokenInfo, Cw721Contract, ExecuteMsg as Cw721ExecuteMsg};

// #[test]
// fn test_instantiate() {
//     let mut deps = mock_dependencies();
//     let msg = InstantiateMsg {
//         kernel_address: MOCK_KERNEL_CONTRACT.to_string(),
//         owner: Some("creator".to_owned()),
//     };
//     let info = mock_info("sender", &[]);
//     let env = mock_env();

//     let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
//     assert_eq!(0, res.messages.len());
// }

// #[test]
// fn test_set_timelock() {
//     let mut deps = mock_dependencies_custom(&[]);
//     let msg = InstantiateMsg {
//         kernel_address: MOCK_KERNEL_CONTRACT.to_string(),
//         owner: Some("creator".to_owned()),
//     };
//     let info = mock_info("sender", &[]);
//     let env = mock_env();

//     instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

//     // Simulate the owner approving the timelock contract as an operator
//     // let approval_msg = Cw721ExecuteMsg::Approve {
//     //     spender: env.contract.address.to_string(),
//     //     token_id: "token1".to_string(),
//     //     expires: None,
//     // };

//     // let approval_info = mock_info(MOCK_TOKEN_OWNER, &[]);
//     // let approval_res = execute(deps.as_mut(), env.clone(), approval_info, ExecuteMsg::Cw721 {
//     //     contract_addr: MOCK_CW721_CONTRACT.to_string(),
//     //     msg: encode_binary(&approval_msg).unwrap(),
//     // }).unwrap();

//     // println!("Approval response: {:?}", approval_res);

//     // println!("Contract Address: {:?}", env.contract.address);


//     // let nft_contract = AndrCW721Contract::default();

//     let set_timelock_msg = ExecuteMsg::SetTimelock {
//         cw721_contract: MOCK_CW721_CONTRACT.to_string(),
//         token_id: "token1".to_string(),
//         unlock_time: 100,
//         recipient: "recipient".to_string(),
//     };

//     let execute_res = execute(deps.as_mut(), env.clone(), info.clone(), set_timelock_msg).unwrap();
//     assert_eq!(execute_res.attributes, vec![("method", "set_timelock")]);

//     // Verify the timelock has been set
//     let query_res: UnlockTimeResponse = from_json(
//         &query(
//             deps.as_ref(),
//             env.clone(),
//             QueryMsg::UnlockTime {
//                 cw721_contract: MOCK_CW721_CONTRACT.to_string(),
//                 token_id: "token1".to_string(),
//             }
//         ).unwrap()
//     ).unwrap();
//     assert_eq!(query_res.unlock_time, env.block.time.seconds() + 100);
// }

// #[test]
// fn test_claim_nft() {
//     let mut deps = mock_dependencies_custom(&[]);
//     let msg = InstantiateMsg {
//         kernel_address: MOCK_KERNEL_CONTRACT.to_string(),
//         owner: Some("creator".to_owned()),
//     };
//     let info = mock_info("sender", &[]);
//     let env = mock_env();

//     instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

//     let set_timelock_msg = ExecuteMsg::SetTimelock {
//         cw721_contract: MOCK_CW721_CONTRACT.to_string(),
//         token_id: "token1".to_string(),
//         unlock_time: 100,
//         recipient: "recipient".to_string(),
//     };

//     execute(deps.as_mut(), env.clone(), info.clone(), set_timelock_msg).unwrap();

//     // Fast forward time
//     let mut env_claim = mock_env();
//     env_claim.block.time = env.block.time.plus_seconds(200);

//     let claim_msg = ExecuteMsg::Claim {
//         cw721_contract: MOCK_CW721_CONTRACT.to_string(),
//         token_id: "token1".to_string(),
//     };

//     let claim_res = execute(deps.as_mut(), env_claim.clone(), info.clone(), claim_msg).unwrap();
//     assert_eq!(claim_res.attributes, vec![("method", "claim_nft")]);

//     // Verify ownership transfer using raw_query
//     let owner_query_msg = to_json_binary(&QueryRequest::<cosmwasm_std::Empty>::Wasm(WasmQuery::Smart {
//         contract_addr: MOCK_CW721_CONTRACT.to_string(),
//         msg: encode_binary(&Cw721QueryMsg::OwnerOf {
//             token_id: "token1".to_string(),
//             include_expired: None,
//         }).unwrap(),
//     })).unwrap();

//     let raw_query_res = deps.querier.raw_query(&owner_query_msg);

//     let owner_response: OwnerOfResponse = from_json(
//         &(raw_query_res.unwrap()).unwrap()
//     ).unwrap();

//     assert_eq!(owner_response.owner, "owner".to_string());
// }

// #[test]
// fn test_locked_nft() {
//     let mut deps = mock_dependencies_custom(&[]);
//     let msg = InstantiateMsg {
//         kernel_address: MOCK_KERNEL_CONTRACT.to_string(),
//         owner: Some("creator".to_owned()),
//     };
//     let info = mock_info("sender", &[]);
//     let env = mock_env();

//     instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

//     let set_timelock_msg = ExecuteMsg::SetTimelock {
//         cw721_contract: MOCK_CW721_CONTRACT.to_string(),
//         token_id: "token1".to_string(),
//         unlock_time: 100,
//         recipient: "recipient".to_string(),
//     };

//     execute(deps.as_mut(), env.clone(), info.clone(), set_timelock_msg).unwrap();

//     // Fast forward time
//     let mut env_claim = mock_env();
//     env_claim.block.time = env.block.time.plus_seconds(20);

//     let claim_msg = ExecuteMsg::Claim {
//         cw721_contract: MOCK_CW721_CONTRACT.to_string(),
//         token_id: "token1".to_string(),
//     };

//     let claim_res = execute(deps.as_mut(), env_claim.clone(), info.clone(), claim_msg).unwrap_err();
//     assert_eq!(claim_res, ContractError::LockedNFT {});
// }

// #[test]
// fn test_query_nft_details() {
//     let mut deps = mock_dependencies_custom(&[]);
//     let msg = InstantiateMsg {
//         kernel_address: MOCK_KERNEL_CONTRACT.to_string(),
//         owner: None,
//     };
//     let info = mock_info("sender", &[]);
//     let env = mock_env();

//     instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

//     let set_timelock_msg = ExecuteMsg::SetTimelock {
//         cw721_contract: MOCK_CW721_CONTRACT.to_string(),
//         token_id: "token1".to_string(),
//         unlock_time: 100,
//         recipient: "recipient".to_string(),
//     };

//     execute(deps.as_mut(), env.clone(), info.clone(), set_timelock_msg).unwrap();

//     let query_msg = QueryMsg::NftDetails {
//         cw721_contract: MOCK_CW721_CONTRACT.to_string(),
//         token_id: "token1".to_string(),
//     };

//     let res: NftDetailsResponse = from_json(
//         &query(deps.as_ref(), env.clone(), query_msg).unwrap()
//     ).unwrap();

//     assert_eq!(res.unlock_time, env.block.time.seconds() + 100);
//     assert_eq!(res.recipient, Addr::unchecked("recipient"));
// }

// #[test]
// fn test_query_unlocktime() {
//     let mut deps = mock_dependencies_custom(&[]);
//     let msg = InstantiateMsg {
//         kernel_address: MOCK_KERNEL_CONTRACT.to_string(),
//         owner: None,
//     };
//     let info = mock_info("sender", &[]);
//     let env = mock_env();

//     instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

//     let set_timelock_msg = ExecuteMsg::SetTimelock {
//         cw721_contract: MOCK_CW721_CONTRACT.to_string(),
//         token_id: "token1".to_string(),
//         unlock_time: 100,
//         recipient: "recipient".to_string(),
//     };

//     execute(deps.as_mut(), env.clone(), info.clone(), set_timelock_msg).unwrap();

//     let query_msg = QueryMsg::UnlockTime {
//         cw721_contract: MOCK_CW721_CONTRACT.to_string(),
//         token_id: "token1".to_string(),
//     };

//     let res: UnlockTimeResponse = from_json(
//         &query(deps.as_ref(), env.clone(), query_msg).unwrap()
//     ).unwrap();

//     assert_eq!(res.unlock_time, env.block.time.seconds() + 100);
// }
