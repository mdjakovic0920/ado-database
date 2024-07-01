use andromeda_std::ado_contract::ADOContract;
pub use andromeda_std::testing::mock_querier::{MOCK_APP_CONTRACT, MOCK_KERNEL_CONTRACT};
use cosmwasm_std::{
    from_json, to_json_binary,
    testing::{mock_env, mock_info, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR},
    Binary, Coin, ContractResult, OwnedDeps, Querier, QuerierResult, QueryRequest, SystemError, SystemResult, WasmQuery, QuerierWrapper,
};
use cw721::{Cw721QueryMsg, OwnerOfResponse};
use andromeda_std::ado_base::InstantiateMsg;
use andromeda_app::app::QueryMsg as AppQueryMsg;

pub const MOCK_CW721_CONTRACT: &str = "cw721_contract";
pub const MOCK_TOKEN_OWNER: &str = "owner";
pub const MOCK_WRAPPED_TOKEN: &str = "wrapped_token";

// Mock dependencies with custom querier
pub fn mock_dependencies_custom(contract_balance: &[Coin]) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(MOCK_CONTRACT_ADDR, contract_balance)]));
    let storage = MockStorage::default();
    let mut deps = OwnedDeps {
        storage,
        api: MockApi::default(),
        querier: custom_querier,
        custom_query_type: std::marker::PhantomData,
    };
    ADOContract::default()
        .instantiate(
            &mut deps.storage,
            mock_env(),
            &deps.api,
            &QuerierWrapper::new(&deps.querier),
            mock_info("sender", &[]),
            InstantiateMsg {
                ado_type: "wrapped-cw721".to_string(),
                ado_version: "test".to_string(),
                kernel_address: MOCK_KERNEL_CONTRACT.to_string(),
                owner: None,
            },
        )
        .unwrap();
    deps
}

pub struct WasmMockQuerier {
    pub base: MockQuerier,
    pub contract_address: String,
    pub tokens_left_to_burn: usize,
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // MockQuerier doesn't support Custom, so we ignore it completely here
        let request: QueryRequest<cosmwasm_std::Empty> = match from_json(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {e}"),
                    request: bin_request.into(),
                })
            }
        };
        self.handle_query(&request)
    }
}

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<cosmwasm_std::Empty>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => {
                match contract_addr.as_str() {
                    MOCK_CW721_CONTRACT => self.handle_token_query(msg),
                    MOCK_APP_CONTRACT => self.handle_app_query(msg),
                    _ => SystemResult::Err(SystemError::UnsupportedRequest {
                        kind: format!("{:?}", request),
                    }),
                }
            }
            _ => SystemResult::Err(SystemError::UnsupportedRequest {
                kind: format!("{:?}", request),
            }),
        }
    }

    fn handle_app_query(&self, msg: &Binary) -> QuerierResult {
        let valid_identifiers = ["e", "b"];
        match from_json(msg).unwrap() {
            AppQueryMsg::ComponentExists { name } => {
                let value = valid_identifiers.contains(&name.as_str());
                SystemResult::Ok(ContractResult::Ok(to_json_binary(&value).unwrap()))
            }
            _ => panic!("Unsupported Query: {msg}"),
        }
    }

    fn handle_token_query(&self, msg: &Binary) -> QuerierResult {
        match from_json(msg).unwrap() {
            Cw721QueryMsg::OwnerOf { token_id, .. } => {
                let res = if token_id == MOCK_WRAPPED_TOKEN {
                    OwnerOfResponse {
                        owner: mock_env().contract.address.to_string(),
                        approvals: vec![],
                    }
                } else {
                    OwnerOfResponse {
                        owner: MOCK_TOKEN_OWNER.to_owned(),
                        approvals: vec![],
                    }
                };
                SystemResult::Ok(ContractResult::Ok(to_json_binary(&res).unwrap()))
            }

            _ => SystemResult::Err(SystemError::UnsupportedRequest {
                kind: format!("{:?}", msg),
            }),
        }
    }

    pub fn new(base: MockQuerier) -> Self {
        WasmMockQuerier {
            base,
            contract_address: mock_env().contract.address.to_string(),
            tokens_left_to_burn: 2,
        }
    }
}