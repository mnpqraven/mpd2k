#![allow(non_snake_case)]

use tokio::sync::Semaphore;

pub static HASH_CONCURRENT_LIMIT: usize = 4;

pub static HASH_SEMAPHORE: Semaphore = Semaphore::const_new(HASH_CONCURRENT_LIMIT);
// 60 fps
pub const TICK_RATE: u64 = 16;
