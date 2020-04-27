use byteorder::{BigEndian, WriteBytesExt};
use error::*;
use rocksdb::{WriteBatch, DB};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use silk_proto::*;
use std::mem;
use std::ops::Range;

const BLOCK_NUM_IDX_KEY_PREFIX: char = 'n';
const BLOCK_HASH_IDX_KEY_PREFIX: char = 'h';
const TX_ID_IDX_KEY_PREFIX: char = 't';
const INDEX_CHECKPOINT_KEY_STR: &str = "index_check_point_key";

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckPoint {
    pub suffix: u64,
    pub offset: u64,
    pub block_num: u64,
    pub block_hash: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilePointer {
    pub suffix: u64,
    pub pos: u64,
    pub len: u64,
}

impl From<(u64, Range<u64>)> for FilePointer {
    fn from((suffix, range): (u64, Range<u64>)) -> Self {
        FilePointer {
            suffix,
            pos: range.start,
            len: range.end - range.start,
        }
    }
}

pub struct BlockIndexInfo<'a> {
    pub fp: FilePointer,
    pub block: &'a Block,
}

pub struct Index {
    db: DB,
}

impl Index {
    pub fn new(db: DB) -> Index {
        Index { db }
    }

    pub fn refresh(&self, info: BlockIndexInfo) -> Result<()> {
        let mut batch = WriteBatch::default();
        let pos = serde_json::to_vec(&info.fp)?;
        let header = info.block.header.as_ref().unwrap();
        let hash = utils::hash::compute_sha256(&utils::proto::marshal(header)?);

        batch.put(&construct_block_hash_key(&hash), &pos);
        batch.put(&construct_block_num_key(header.number), &pos);

        let check_point = CheckPoint {
            suffix: info.fp.suffix,
            offset: info.fp.pos + info.fp.len,
            block_hash: hash.to_vec(),
            block_num: header.number,
        };
        let cp = serde_json::to_vec(&check_point)?;
        batch.put(INDEX_CHECKPOINT_KEY_STR.as_bytes(), &cp);

        // TODO: record tx id

        self.db.write(batch)?;
        Ok(())
    }

    pub fn get_check_point(&self) -> Result<Option<CheckPoint>> {
        self.get(INDEX_CHECKPOINT_KEY_STR.as_ref())
    }

    pub fn get_fp_by_number(&self, num: u64) -> Result<Option<FilePointer>> {
        let key = construct_block_num_key(num);
        self.get(key.as_slice())
    }

    pub fn get_fp_by_hash(&self, hash: &[u8]) -> Result<Option<FilePointer>> {
        let key = construct_block_hash_key(hash);
        self.get(key.as_slice())
    }

    fn get<T>(&self, key: &[u8]) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        match self.db.get(key)? {
            Some(ref dbv) => Ok(Some(serde_json::from_slice(dbv)?)),
            None => Ok(None),
        }
    }
}

fn construct_block_num_key(block_num: u64) -> Vec<u8> {
    let mut v:Vec<u8> = Vec::with_capacity(1 + 8);
    v.write_u8(BLOCK_NUM_IDX_KEY_PREFIX as u8);
    v.write_u64::<BigEndian>(block_num);
    v
}

fn construct_block_hash_key(block_hash: &[u8]) -> Vec<u8> {
    let mut v = vec![BLOCK_HASH_IDX_KEY_PREFIX as u8];
    v.append(&mut block_hash.to_vec());
    v
}

fn construct_tx_hash_key(tx: String) -> Vec<u8> {
    let mut v = vec![TX_ID_IDX_KEY_PREFIX as u8];
    v.append(&mut tx.into_bytes());
    v
}

#[cfg(test)]
mod tests {
    use crate::index::{
        construct_block_hash_key, construct_block_num_key, BlockIndexInfo, FilePointer, Index,
    };
    use rocksdb::DB;
    use silk_proto::*;
    use tempfile::TempDir;

    #[test]
    fn test_construct() {
        let s = construct_block_num_key(12);
        println!("{:?}", s);
        let s1 = construct_block_hash_key("hash1".as_bytes());
        println!("{:?}", s1);
    }

    #[test]
    fn test_index_block() {
        let temp_dir = TempDir::new().unwrap();
        let db = DB::open_default(temp_dir.path().to_str().unwrap()).unwrap();
        let index = Index::new(db);

        let cp = index.get_check_point().unwrap();
        assert!(cp.is_none());

        let header1 = BlockHeader{
            number: 1,
            previous_hash: "data_hash1".to_string().into_bytes(),
            data_hash: vec![]
        };
        let block_hash1 = utils::hash::compute_sha256(utils::proto::marshal(&header1).unwrap().as_slice());

        let block1 = Block{
            header: Some(header1),
            data: None,
            metadata: None
        };

        let bi1 = BlockIndexInfo {
            fp: FilePointer {
                suffix: 1,
                pos: 0,
                len: 1024,
            },
            block: &block1,
        };

        let res1 = index.refresh(bi1);
        assert!(res1.is_ok());

        let cp = index.get_check_point().unwrap().unwrap();
        println!("{:?}", cp);
        assert_eq!(cp.suffix, 1);
        assert_eq!(cp.block_num, 1);
        assert_eq!(cp.block_hash[..], block_hash1[..]);

        let mut header2= BlockHeader{
            number: 2,
            previous_hash: block_hash1.into_vec(),
            data_hash: "data_hash2".to_string().into_bytes()
        };
        let block_hash2 = utils::hash::compute_sha256(utils::proto::marshal(&header2).unwrap().as_slice());
        let mut block2 = Block{
            header: Some(header2.clone()),
            data: None,
            metadata: None
        };
        let block2 = BlockIndexInfo {
            fp: FilePointer {
                suffix: 1,
                pos: 1025,
                len: 43,
            },
            block: &block2,
        };

        let res2 = index.refresh(block2);
        assert!(res2.is_ok());

        let cp2 = index.get_check_point().unwrap().unwrap();
        println!("{:?}", cp2);
        assert_eq!(cp2.suffix, 1);
        assert_eq!(cp2.block_num, 2);
        assert_eq!(cp2.block_hash[..], block_hash2[..]);

        let blk1 = index.get_fp_by_number(1).unwrap().unwrap();
        assert_eq!(blk1.suffix, 1);
        assert_eq!(blk1.pos, 0);
        assert_eq!(blk1.len, 1024);

        let blk2 = index
            .get_fp_by_hash(&utils::hash::compute_sha256(
                utils::proto::marshal(&header2).unwrap().as_slice(),
            ))
            .unwrap()
            .unwrap();
        assert_eq!(blk2.suffix, 1);
        assert_eq!(blk2.pos, 1025);
        assert_eq!(blk2.len, 43);
    }
}
