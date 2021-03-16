use byteorder::{BigEndian, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::mem::size_of;

pub const BLOCK_NUM_IDX_KEY_PREFIX: u8 = b'n';
pub const BLOCK_HASH_IDX_KEY_PREFIX: u8 = b'h';
pub const TX_ID_IDX_KEY_PREFIX: u8 = b't';
pub const INDEX_CHECKPOINT_KEY_STR: &str = "index_check_point_key";

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckPoint {
    pub suffix: u64,
    pub offset: u64,
    pub block_num: u64,
    pub block_hash: Vec<u8>,
    pub previous_block_hash: Vec<u8>,
}

pub fn construct_block_num_key(block_num: u64) -> Vec<u8> {
    let mut v: Vec<u8> = Vec::with_capacity(1 + size_of::<u64>());
    v.push(BLOCK_NUM_IDX_KEY_PREFIX);
    v.write_u64::<BigEndian>(block_num).unwrap();
    v
}

pub fn construct_block_hash_key(block_hash: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(1 + block_hash.len());
    v.push(BLOCK_HASH_IDX_KEY_PREFIX);
    v.append(&mut block_hash.to_vec());
    v
}

pub fn construct_tx_hash_key(tx_id: &str) -> Vec<u8> {
    let mut v = Vec::with_capacity(1 + tx_id.len());
    v.push(TX_ID_IDX_KEY_PREFIX);
    v.append(&mut tx_id.as_bytes().to_vec());
    v
}

pub fn construct_check_point_key() -> Vec<u8> {
    INDEX_CHECKPOINT_KEY_STR.as_bytes().to_vec()
}

#[cfg(test)]
mod tests {
    use crate::keys::{construct_block_hash_key, construct_block_num_key, construct_tx_hash_key};
    use byteorder::{BigEndian, WriteBytesExt};
    use std::mem::size_of;

    #[test]
    fn test() {
        let k = construct_block_num_key(12100000000);
        assert_eq!(vec![110, 0, 0, 0, 2, 209, 55, 89, 0], k);

        let k = construct_block_hash_key(&vec![1, 2, 3]);
        assert_eq!(vec![104, 1, 2, 3], k);

        let k = construct_tx_hash_key("abc");
        assert_eq!(vec![116, 97, 98, 99], k)
    }
}
