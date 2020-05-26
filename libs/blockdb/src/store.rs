use crate::cp::{
    construct_block_hash_key, construct_block_num_key, construct_check_point_key,
    construct_tx_hash_key, CheckPoint,
};
use crate::{BlockIterator, BlockStore};
use error::*;
use rocksdb::WriteBatch;
use serde::de::DeserializeOwned;
use silk_proto::{Block, BlockchainInfo, Envelope, Proposal, Transaction, TxValidationCode};
use std::path::PathBuf;

pub struct Store {
    db: rocksdb::DB,
}

impl Store {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let path = path.join("blk_store");
        let db = rocksdb::DB::open_default(
            path.to_str()
                .ok_or_else(|| "get path str error".to_string())?,
        )?;
        Ok(Store { db })
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

impl BlockStore for Store {
    fn add_block(&mut self, block: &Block) -> Result<()> {
        let check_point: Option<CheckPoint> = self.get(&construct_check_point_key())?;
        let mut batch = WriteBatch::default();

        if let (Some(header), Some(data)) = (block.header.clone(), block.data.clone()) {
            if check_point.is_none() {
                if header.number != 0 {
                    return Err(from_str("block not is genesis block"));
                }
            } else {
                let mut check_point = check_point.unwrap();
                // the block has been saved
                if check_point.block_num >= header.number {
                    return Ok(());
                }

                // lose blocks
                if check_point.block_num + 1 < header.number {
                    return Err(from_str("block number > checkpoint number + 1"));
                }

                let hash = utils::hash::compute_sha256(&utils::proto::marshal(&header)?);

                check_point.block_num = header.number;
                check_point.block_hash = hash.to_vec();
                let cp = serde_json::to_vec(&check_point)?;
                batch.put(&construct_check_point_key(), &cp);
                batch.put(
                    &construct_block_hash_key(&hash),
                    &utils::proto::marshal(block)?,
                );
                batch.put(&construct_block_num_key(header.number), &hash);

                // record txs id mapping block hash
                for evn in data.data {
                    let tx = utils::proto::unmarshal::<Transaction>(&evn)?;
                    let signed_proposal = tx.signed_proposal.unwrap();
                    let proposal =
                        utils::proto::unmarshal::<Proposal>(&signed_proposal.proposal_bytes)?;
                    let tx_header = proposal.header.unwrap();
                    batch.put(&construct_tx_hash_key(tx_header.tx_id), &hash);
                }
                self.db.write(batch)?;
            }
            Ok(())
        } else {
            Err(from_str("block header or data is null"))
        }
    }

    fn get_blockchain_info(&self) -> Result<BlockchainInfo> {
        unimplemented!()
    }

    fn retrieve_blocks(&self, _start_num: u64) -> Result<Box<dyn BlockIterator>> {
        unimplemented!()
    }

    fn retrieve_block_by_hash(&self, _block_hash: &[u8]) -> Result<Block> {
        unimplemented!()
    }

    fn retrieve_block_by_number(&self, _block_num: u64) -> Result<Block> {
        unimplemented!()
    }

    fn retrieve_tx_by_id(&self, _tx_id: String) -> Result<Envelope> {
        unimplemented!()
    }

    fn retrieve_tx_by_blocknum_txnum(&self, _block_num: u64, _tx_num: u64) -> Result<Envelope> {
        unimplemented!()
    }

    fn retrieve_block_by_txid(&self, _tx_id: String) -> Result<Block> {
        unimplemented!()
    }

    fn retrieve_tx_validationcode_by_txid(&self, _tx_id: String) -> Result<TxValidationCode> {
        unimplemented!()
    }

    fn shutdown() {
        unimplemented!()
    }
}
