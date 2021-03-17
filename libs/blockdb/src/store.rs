use crate::keys;
use crate::BlockStore;
use error::*;
use rocksdb::WriteBatch;
use serde::de::DeserializeOwned;
use silk_proto::{
    tx_validation_code_from, Block, BlockchainInfo, Transaction, TxIdIndexValProto,
    TxValidationCode,
};
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

    fn get_tx_validation_code_by_txid(&self, tx_id: &str) -> Result<TxIdIndexValProto> {
        let tx_index_val = self.db.get(&keys::construct_tx_hash_key(tx_id))?;

        match tx_index_val {
            Some(tx_index_val) => {
                let index_val: TxIdIndexValProto = utils::proto::unmarshal(&tx_index_val)?;
                Ok(index_val)
            }
            None => Ok(TxIdIndexValProto {
                block_hash: vec![],
                tx_validation_code: TxValidationCode::NilEnvelope as i32,
            }),
        }
    }
}

impl BlockStore for Store {
    fn add_block(&mut self, block: &Block) -> Result<()> {
        let check_point: Option<keys::CheckPoint> = self.get(&keys::construct_check_point_key())?;
        let mut batch = WriteBatch::default();

        if let (Some(header), Some(data)) = (block.header.clone(), block.data.clone()) {
            let mut check_point = match check_point {
                Some(cp) => {
                    // the block has been saved
                    if cp.block_num >= header.number {
                        return Ok(());
                    }

                    // lose blocks
                    if cp.block_num + 1 < header.number {
                        return Err(from_str("block number > checkpoint number + 1"));
                    }
                    cp
                }
                None => keys::CheckPoint {
                    suffix: 0,
                    offset: 0,
                    block_num: 0,
                    block_hash: vec![],
                    previous_block_hash: vec![],
                    tx_total_count: 0,
                },
            };
            let hash = utils::hash::compute_sha256(&utils::proto::marshal(&header)?);

            check_point.block_num = header.number;
            check_point.block_hash = hash.to_vec();
            check_point.previous_block_hash = header.previous_hash;
            check_point.tx_total_count += data.data.len() as u128;

            let cp = serde_json::to_vec(&check_point)?;
            batch.put(&keys::construct_check_point_key(), &cp);

            // mapping block hash -> block
            batch.put(
                &keys::construct_block_hash_key(&hash),
                &utils::proto::marshal(block)?,
            );

            // mapping block num -> block hash
            batch.put(&keys::construct_block_num_key(header.number), &hash);

            // record txs id mapping block hash
            for evn in data.data {
                let (_, tx_header) = utils::utils::get_tx_header_from_data(&evn)?;

                // mapping tx_id -> TxIdIndexValProto
                let index_val = TxIdIndexValProto {
                    block_hash: hash.to_vec(),
                    tx_validation_code: TxValidationCode::Valid as i32,
                };
                debug!("tx: {:?} index value: {:?}", tx_header.tx_id, index_val);
                batch.put(
                    &keys::construct_tx_hash_key(&tx_header.tx_id),
                    &utils::proto::marshal(&index_val)?,
                );
            }

            self.db.write(batch)?;
            self.db.flush()?;
            Ok(())
        } else {
            Err(from_str("block header or data is null"))
        }
    }

    fn get_blockchain_info(&self) -> Result<BlockchainInfo> {
        let check_point: Option<keys::CheckPoint> = self.get(&keys::construct_check_point_key())?;
        match check_point {
            Some(cp) => Ok(BlockchainInfo {
                height: cp.block_num,
                current_block_hash: cp.block_hash,
                previous_block_hash: cp.previous_block_hash,
            }),
            None => Ok(BlockchainInfo {
                height: 0,
                current_block_hash: vec![],
                previous_block_hash: vec![],
            }),
        }
    }

    fn retrieve_blocks(
        &self,
        start_num: u64,
    ) -> Result<Box<dyn Iterator<Item = Result<Option<Block>>>>> {
        Ok(Box::new(BlockIterator { curr: start_num }))
    }

    fn retrieve_block_by_hash(&self, block_hash: &[u8]) -> Result<Option<Block>> {
        let blk_bytes = self.db.get(&keys::construct_block_hash_key(block_hash))?;
        if blk_bytes.is_none() {
            return Ok(None);
        }

        let block = utils::proto::unmarshal(&blk_bytes.unwrap())?;
        Ok(Some(block))
    }

    fn retrieve_block_by_number(&self, block_num: u64) -> Result<Option<Block>> {
        let num = if block_num == std::u64::MAX {
            let check_point: Option<keys::CheckPoint> =
                self.get(&keys::construct_check_point_key())?;
            match check_point {
                Some(cp) => cp.block_num,
                None => 0,
            }
        } else {
            block_num
        };

        let hash = self.db.get(&keys::construct_block_num_key(num))?;
        match hash {
            Some(hash) => self.retrieve_block_by_hash(&hash),
            None => Ok(None),
        }
    }

    fn retrieve_tx_by_id(&self, tx_id: &str) -> Result<Option<Transaction>> {
        let blk = self.retrieve_block_by_txid(tx_id)?;

        match blk {
            Some(blk) => {
                for env in blk.data.unwrap().data {
                    let (tx, header) = utils::utils::get_tx_header_from_data(&env)?;
                    if header.tx_id.eq(&tx_id) {
                        return Ok(Some(tx));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    fn retrieve_tx_by_blocknum_txnum(
        &self,
        block_num: u64,
        tx_num: u64,
    ) -> Result<Option<Transaction>> {
        let blk = self.retrieve_block_by_number(block_num)?;
        if blk.is_none() {
            return Ok(None);
        }

        match blk {
            Some(blk) => {
                let txs = blk.data.unwrap();
                let tx_bytes = txs.data.get(tx_num as usize);
                if tx_bytes.is_none() {
                    return Ok(None);
                }

                let (tx, _) = utils::utils::get_tx_header_from_data(tx_bytes.unwrap())?;
                Ok(Some(tx))
            }
            None => Ok(None),
        }
    }

    fn retrieve_block_by_txid(&self, tx_id: &str) -> Result<Option<Block>> {
        let tx_index_val = self.get_tx_validation_code_by_txid(tx_id).unwrap();

        if tx_index_val.block_hash.is_empty() {
            return Ok(None);
        }

        self.retrieve_block_by_hash(&tx_index_val.block_hash)
    }

    fn retrieve_tx_validation_code_by_txid(&self, tx_id: &str) -> Result<TxValidationCode> {
        self.get_tx_validation_code_by_txid(tx_id)
            .map(|v| tx_validation_code_from(v.tx_validation_code))
    }
}

// ResultsIterator iterates over query results
pub struct BlockIterator {
    curr: u64,
}

impl Iterator for BlockIterator {
    type Item = Result<Option<Block>>;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use crate::store::Store;
    use crate::BlockStore;
    use error::*;
    use silk_proto::*;
    use tempfile::TempDir;

    fn init() -> Result<Store> {
        let temp_dir = TempDir::new().unwrap();
        let mut store = Store::open(temp_dir.into_path())?;

        store.add_block(&create_block(
            0,
            vec![],
            vec![create_tx("tx_0".to_string())?],
        ))?;

        for i in 1..=100 {
            let info = store.get_blockchain_info()?;
            let block = create_block(
                i as u64,
                info.current_block_hash,
                vec![create_tx(format!("tx_{:}", i))?],
            );
            store.add_block(&block)?;
        }
        Ok(store)
    }

    fn create_block(num: u64, prev_hash: Vec<u8>, txs: Vec<Transaction>) -> Block {
        let data: Vec<Vec<u8>> = txs
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

    #[test]
    fn test_get_blockchain_info() {
        let store = init().unwrap();
        let info = store.get_blockchain_info().unwrap();
        assert_eq!(info.height, 100);
        println!("{:?}", info);
    }

    #[test]
    fn test_retrieve_blocks() {}

    #[test]
    fn test_retrieve_block_by_hash() {
        let store = init().unwrap();
        let info = store.get_blockchain_info().unwrap();

        let blk = store
            .retrieve_block_by_hash(&info.current_block_hash)
            .unwrap()
            .unwrap();
        println!("{:?}", blk);
        let mut hash = blk.header.unwrap().previous_hash;
        while hash.len() > 0 {
            let blk = store.retrieve_block_by_hash(&hash).unwrap().unwrap();
            let header = blk.header.unwrap();
            println!(
                "{:?} : {:?} {:?}",
                header.number,
                header.data_hash.len(),
                header.previous_hash.len()
            );
            hash = header.previous_hash;
        }
    }

    #[test]
    fn test_retrieve_block_by_number() {
        let store = init().unwrap();

        for i in 0..=100 {
            let blk = store.retrieve_block_by_number(i).unwrap().unwrap();
            let header = blk.header.unwrap();
            println!(
                "{:?} : {:?} {:?}",
                header.number,
                header.data_hash.len(),
                header.previous_hash
            );
            assert_eq!(i, header.number)
        }

        let blk1000 = store.retrieve_block_by_number(1000).unwrap();
        assert!(blk1000.is_none())
    }

    #[test]
    fn test_retrieve_tx_by_id() {
        let store = init().unwrap();

        let tx = store.retrieve_tx_by_id("tx1").unwrap();
        assert!(tx.is_none());

        let tx = store.retrieve_tx_by_id("tx_12").unwrap().unwrap();
        println!("{:?}", tx);

        let signed_proposal = tx.signed_proposal.unwrap();

        let proposal =
            utils::proto::unmarshal::<Proposal>(&signed_proposal.proposal_bytes).unwrap();
        println!("{:?}", proposal)
    }

    #[test]
    fn test_retrieve_tx_by_blocknum_txnum() {
        let store = init().unwrap();

        let tx = store.retrieve_tx_by_blocknum_txnum(99, 2).unwrap();
        assert!(tx.is_none());

        let tx = store.retrieve_tx_by_blocknum_txnum(100, 1).unwrap();
        assert!(tx.is_none());

        let tx = store.retrieve_tx_by_blocknum_txnum(10, 0).unwrap().unwrap();
        println!("{:?}", tx)
    }

    #[test]
    fn test_retrieve_block_by_txid() {
        let store = init().unwrap();

        let blk = store.retrieve_block_by_txid("tx1").unwrap();
        assert!(blk.is_none());

        let blk = store.retrieve_block_by_txid("tx_99").unwrap().unwrap();
        println!("{:?}", blk);
    }

    #[test]
    fn test_retrieve_tx_validation_code_by_txid() {
        let store = init().unwrap();

        let code = store.retrieve_tx_validation_code_by_txid("tx1").unwrap();
        assert_eq!(code, TxValidationCode::NilEnvelope);

        for i in 0..=100 {
            let code = store
                .retrieve_tx_validationcode_by_txid(&format!("tx_{:}", i))
                .unwrap();
            assert_eq!(code, TxValidationCode::Valid)
        }
    }
}
