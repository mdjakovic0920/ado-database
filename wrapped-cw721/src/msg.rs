use cosmwasm_schema::{cw_serde, QueryResponses};
use cw721::Cw721ReceiveMsg;
use andromeda_std::{
    andr_instantiate, andr_exec, andr_query,
    amp::AndrAddr,
};
use andromeda_non_fungible_tokens::cw721::TokenExtension;

#[andr_instantiate]
#[cw_serde]
pub struct InstantiateMsg {}

#[andr_exec]
#[cw_serde]
pub enum ExecuteMsg {
    // SetWrappedTokenAddress{
    //     token_address: AndrAddr,
    // },
    ReceiveNft(Cw721ReceiveMsg),
}

#[cw_serde]
pub enum Cw721HookMsg {
    MintWrappedNft {
        wrapped_token: AndrAddr,
        wrapped_token_id: String,
        wrapped_token_owner: String,
        wrapped_token_uri: Option<String>,
        wrapped_token_extension: TokenExtension,
        unwrappable: bool,
    },
    UnwrapNft {
        recipient: AndrAddr,
        wrapped_token: AndrAddr,
        wrapped_token_id: String,
    },
}

#[andr_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(OriginCw721Response)]
    OriginCw721 {
        wrapped_token: AndrAddr,
        wrapped_token_id: String,
    },

    #[returns(WrappedCw721Response)]
    WrappedCw721 {
        origin_token: AndrAddr,
        origin_token_id: String,
    },

    #[returns(IsUnwrappableResponse)]
    IsUnwrappable {
        wrapped_token: AndrAddr,
        wrapped_token_id: String,
    },
}

#[cw_serde]
pub struct OriginCw721Response {
    pub origin_token: AndrAddr,
    pub origin_token_id: String,
}

#[cw_serde]
pub struct WrappedCw721Response {
    pub wrapped_token: AndrAddr,
    pub wrapped_token_id: String,
}

#[cw_serde]
pub struct IsUnwrappableResponse {
    pub is_unwrappable: bool,
}
