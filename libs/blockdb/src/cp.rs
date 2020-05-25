use byteorder::{BigEndian, WriteBytesExt};
use serde::{Deserialize, Serialize};


pub const BLOCK_NUM_IDX_KEY_PREFIX: char = 'n';
pub const BLOCK_HASH_IDX_KEY_PREFIX: char = 'h';
pub const TX_ID_IDX_KEY_PREFIX: char = 't';
pub const INDEX_CHECKPOINT_KEY_STR: &str = "index_check_point_key";

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckPoint {
    pub suffix: u64,
    pub offset: u64,
    pub block_num: u64,
    pub block_hash: Vec<u8>,
}

pub fn construct_block_num_key(block_num: u64) -> Vec<u8> {
    let mut v: Vec<u8> = Vec::with_capacity(1 + 8);
    v.write_u8(BLOCK_NUM_IDX_KEY_PREFIX as u8);
    v.write_u64::<BigEndian>(block_num);
    v
}

pub fn construct_block_hash_key(block_hash: &[u8]) -> Vec<u8> {
    let mut v = vec![BLOCK_HASH_IDX_KEY_PREFIX as u8];
    v.append(&mut block_hash.to_vec());
    v
}

pub fn construct_tx_hash_key(tx: String) -> Vec<u8> {
    let mut v = vec![TX_ID_IDX_KEY_PREFIX as u8];
    v.append(&mut tx.into_bytes());
    v
}

pub fn construct_check_point_key() -> Vec<u8> {
    INDEX_CHECKPOINT_KEY_STR.as_bytes().to_vec()
}
