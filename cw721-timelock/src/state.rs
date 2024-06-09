use cosmwasm_std::{Addr};
// use cosmwasm_storage::{Bucket, ReadonlyBucket, singleton, singleton_read, Singleton};
// use schemars::JsonSchema;
use cosmwasm_schema::cw_serde;
use cw_storage_plus::Map;

use andromeda_std::common::milliseconds::MillisecondsExpiration;

// pub const CONFIG: Item<State> = Item::new("config");
pub const TIMELOCKS: Map<&str, TimelockInfo> = Map::new("timelocks");

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub struct State {
//     pub owner: Addr,
// }

#[cw_serde]
pub struct TimelockInfo {
    pub unlock_time: MillisecondsExpiration,
    pub recipient: Addr,
}

// pub fn config(storage: &mut dyn cosmwasm_std::Storage) -> Singleton<State> {
//     singleton(storage, CONFIG_KEY)
// }

// pub fn config_read(storage: &dyn cosmwasm_std::Storage) -> Singleton<State> {
//     singleton_read(storage, CONFIG_KEY)
// }

// pub fn timelocks<'a>(storage: &'a mut dyn cosmwasm_std::Storage) -> Bucket<'a, TimelockInfo> {
//     Bucket::new(TIMELOCKS_KEY, storage)
// }

// pub fn timelocks_read<'a>(storage: &'a dyn cosmwasm_std::Storage) -> ReadonlyBucket<'a, TimelockInfo> {
//     ReadonlyBucket::new(TIMELOCKS_KEY, storage)
// }