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
                check_point.previous_block_hash = header.previous_hash;
                let cp = serde_json::to_vec(&check_point)?;
                batch.put(&construct_check_point_key(), &cp);
                batch.put(
                    &construct_block_hash_key(&hash),
                    &utils::proto::marshal(block)?,
                );
                batch.put(&construct_block_num_key(header.number), &hash);

                // record txs id mapping block hash
                for evn in data.data {
                    let (_, tx_header) = utils::utils::get_tx_header_from_data(&evn)?;
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
        let check_point: Option<CheckPoint> = self.get(&construct_check_point_key())?;
        match check_point {
            Some(cp) => {
                Ok(BlockchainInfo {
                    height: cp.block_num,
                    current_block_hash: cp.block_hash,
                    previous_block_hash: cp.previous_block_hash,
                })
            }
            None => {
                Ok(BlockchainInfo {
                    height: 0,
                    current_block_hash: vec![],
                    previous_block_hash: vec![],
                })
            }
        }
    }

    fn retrieve_blocks(&self, _start_num: u64) -> Result<Box<dyn BlockIterator>> {
        unimplemented!()
    }

    fn retrieve_block_by_hash(&self, block_hash: &[u8]) -> Result<Option<Block>> {
        let blk_bytes = self.db.get(&construct_block_hash_key(block_hash))?;
        if blk_bytes.is_none() {
            return Ok(None);
        }

        let block = utils::proto::unmarshal(&blk_bytes.unwrap())?;
        Ok(Some(block))
    }

    fn retrieve_block_by_number(&self, block_num: u64) -> Result<Block> {
        let check_point: Option<CheckPoint> = self.get(&construct_check_point_key())?;
        match check_point {
            Some(cp) => {
                let mut num = block_num;
                if block_num > cp.block_num {
                    num = cp.block_num
                }
                let blk_bytes = self.db.get(&construct_block_num_key(num))?.unwrap();
                let blk = utils::proto::unmarshal(&blk_bytes)?;
                Ok(blk)
            }
            None => {
                Ok(Block {
                    header: None,
                    data: None,
                    metadata: None,
                })
            }
        }
    }

    fn retrieve_tx_by_id(&self, tx_id: String) -> Result<Option<Transaction>> {
        let blk = self.retrieve_block_by_txid(tx_id.clone())?;
        if blk.is_none() {
            return Ok(None);
        }
        let blk = blk.unwrap();
        for env in blk.data.unwrap().data {
            let (tx, header) = utils::utils::get_tx_header_from_data(&env)?;
            if header.tx_id.eq(&tx_id) {
                return Ok(Some(tx));
            }
        }
        Ok(None)
    }

    fn retrieve_tx_by_blocknum_txnum(&self, block_num: u64, tx_num: u64) -> Result<Option<Transaction>> {
        let blk = self.retrieve_block_by_number(block_num)?;
        if blk.data.is_none() {
            return Ok(None);
        }
        let txs = blk.data.unwrap();
        let tx_bytes = txs.data.get(tx_num as usize);
        if tx_bytes.is_none() {
            return Ok(None);
        }

        let (tx , _) = utils::utils::get_tx_header_from_data(tx_bytes.unwrap())?;
        Ok(Some(tx))
    }

    fn retrieve_block_by_txid(&self, tx_id: String) -> Result<Option<Block>> {
        let hash = self.db.get(&construct_tx_hash_key(tx_id))?;
        if hash.is_none() {
            return Ok(None);
        }
        let hash = hash.unwrap();
        let blk_bytes = self.db.get(&construct_block_hash_key(&hash))?.unwrap();
        let blk = utils::proto::unmarshal(&blk_bytes)?;
        Ok(Some(blk))
    }

    fn retrieve_tx_validationcode_by_txid(&self, tx_id: String) -> Result<TxValidationCode> {
        unimplemented!()
    }
}


#[cfg(test)]
mod tests {
    use silk_proto::*;
    use error::*;
    use crate::store::Store;
    use tempfile::TempDir;
    use crate::BlockStore;

    fn init() -> Result<Store> {
        let temp_dir = TempDir::new().unwrap();
        let mut store = Store::open(temp_dir.into_path())?;

        store.add_block(&create_block(0, vec![], vec![create_tx("txo".to_string())?]))?;

        for i in 1..=100 {
            let info = store.get_blockchain_info()?;
            let block = create_block(i as u64, info.current_block_hash, vec![create_tx(format!("tx_{:}", i))?]);
            store.add_block(&block);
        }
        Ok(store)
    }

    #[test]
    fn test_get_blockchain_info() {
        let store = init().unwrap();
    }
    fn create_block(num: u64, prev_hash: Vec<u8>, txs: Vec<Transaction>) -> Block {
        let data = txs
            .iter()
            .map(|t| utils::proto::marshal(t).unwrap())
            .collect();



        Block {
            header: Some(BlockHeader {
                number: num,
                previous_hash: prev_hash,
                data_hash: utils::hash::compute_vec_sha256(&data).to_vec(),
            }),
            data: Some(BlockData { data }),
            metadata: None,
        }
    }
    fn create_tx(txid: String) -> Result<Transaction> {
        let payload = ContractProposalPayload {
            contract_id: None,
            input: None,
            transient_map: Default::default(),
            timeout: 0,
        };

        let proposal = Proposal {
            header: Some(Header {
                header_type: HeaderType::Invoke as i32,
                version: 0,
                timestamp: Some(utils::time::timestamp()),
                channel_id: "chain_id".to_string(),
                tx_id: txid,
                tls_cert_hash: vec![],
                creator: vec![],
                nonce: utils::random::get_random_nonce(),
            }),
            payload: utils::proto::marshal(&payload)?,
        };
        let sp = SignedProposal {
            proposal_bytes: utils::proto::marshal(&proposal)?,
            signature: vec![],
        };

        let payload = ProposalResponsePayload {
            results: vec![],
            events: vec![],
        };
        let proposal_response = ProposalResponse {
            version: 0,
            timestamp: None,
            response: None,
            payload: utils::proto::marshal(&payload)?,
            endorsement: None,
        };

        let tx = Transaction {
            signed_proposal: Some(sp),
            response: vec![proposal_response],
        };
        Ok(tx)
    }
}
