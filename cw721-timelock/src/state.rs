use cosmwasm_std::{Addr};
use cosmwasm_schema::cw_serde;
use cw_storage_plus::Map;
use andromeda_std::common::milliseconds::MillisecondsExpiration;

pub const TIMELOCKS: Map<&str, TimelockInfo> = Map::new("timelocks");

#[cw_serde]
pub struct TimelockInfo {
    pub unlock_time: MillisecondsExpiration,
    pub recipient: Addr,
}
