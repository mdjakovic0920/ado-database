#![cfg(all(not(target_arch = "wasm32"), feature = "testing"))]

use crate::contract::{execute, instantiate, query};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use andromeda_std::ado_base::rates::Rate;
use andromeda_std::ado_base::rates::RatesMessage;
use andromeda_std::amp::messages::AMPPkt;

use andromeda_std::amp::AndrAddr;
use andromeda_std::common::denom::Asset;
use andromeda_std::common::expiration::Expiry;
use andromeda_testing::{
    mock::MockApp, mock_ado, mock_contract::ExecuteResult, MockADO, MockContract,
};
use cosmwasm_std::{Addr, Empty, Uint128};
use cw_multi_test::{Contract, ContractWrapper, Executor};

pub struct MockCw721Timelock(Addr);
mock_ado!(MockCw721Timelock, ExecuteMsg, QueryMsg);

impl MockCw721Timelock {
    pub fn instantiate(
        code_id: u64,
        sender: Addr,
        app: &mut MockApp,
        kernel_address: impl Into<String>,
        owner: Option<String>,
    ) -> MockCw721Timelock {
        let msg = mock_timelock_instantiate_msg(kernel_address.into(), owner);
        let addr = app
            .instantiate_contract(
                code_id,
                sender.clone(),
                &msg,
                &[],
                "Timelock Contract",
                Some(sender.to_string()),
            )
            .unwrap();
            MockCw721Timelock(addr)
    }
}

pub fn mock_andromeda_timelock() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new_with_empty(execute, instantiate, query);
    Box::new(contract)
}

pub fn mock_timelock_instantiate_msg(
    kernel_address: String,
    owner: Option<String>,
) -> InstantiateMsg {
    InstantiateMsg {}
}