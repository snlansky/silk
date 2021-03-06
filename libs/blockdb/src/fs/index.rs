use error::*;
use rocksdb::{WriteBatch, DB};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use silk_proto::*;

use crate::keys::{
    construct_block_hash_key, construct_block_num_key, construct_check_point_key, CheckPoint,
};
use std::ops::Range;

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
            previous_block_hash: header.previous_hash.clone(),
            // TODO: set tx total count
            tx_total_count: 0,
        };
        let cp = serde_json::to_vec(&check_point)?;
        batch.put(&construct_check_point_key(), &cp);

        // TODO: record tx id

        self.db.write(batch)?;
        Ok(())
    }

    pub fn get_check_point(&self) -> Result<Option<CheckPoint>> {
        self.get(&construct_check_point_key())
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

#[cfg(test)]
mod tests {
    use crate::fs::index::{BlockIndexInfo, FilePointer, Index};
    use crate::keys::{construct_block_hash_key, construct_block_num_key};
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

        let header1 = BlockHeader {
            number: 1,
            previous_hash: "data_hash1".to_string().into_bytes(),
            data_hash: vec![],
        };
        let block_hash1 =
            utils::hash::compute_sha256(utils::proto::marshal(&header1).unwrap().as_slice());

        let block1 = Block {
            header: Some(header1),
            data: None,
            metadata: None,
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

        let header2 = BlockHeader {
            number: 2,
            previous_hash: block_hash1.into_vec(),
            data_hash: "data_hash2".to_string().into_bytes(),
        };
        let block_hash2 =
            utils::hash::compute_sha256(utils::proto::marshal(&header2).unwrap().as_slice());
        let block2 = Block {
            header: Some(header2.clone()),
            data: None,
            metadata: None,
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
