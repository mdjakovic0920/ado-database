// use andromeda_non_fungible_tokens::cw721::TransferAgreement;
use andromeda_std::amp::AndrAddr;
// use cosmwasm_std::Storage;
use cw_storage_plus::Map;
use cosmwasm_schema::cw_serde;
// use cw721::{ContractInfoResponse, Cw721, Expiration};

pub const WRAPPED_TOKEN_ADDRESS: Map<&str, AndrAddr> = Map::new("wrapped_token_address");
pub const WRAPPED_INFO: Map<&str, WrappedInfo> = Map::new("origin_id");
pub const ORIGIN_INFO: Map<&str, OriginInfo> = Map::new("wrapped_id");

#[cw_serde]
pub struct WrappedInfo {
    pub wrapped_token: AndrAddr,
    pub wrapped_token_id: String,
    pub unwrappable: bool,
}

#[cw_serde]
pub struct OriginInfo {
    pub origin_token: AndrAddr,
    pub origin_token_id: String,
    pub unwrappable: bool,
}

