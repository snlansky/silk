use rocksdb::DB;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use error::*;
use silk_proto::*;
use crate::fs::writer::BlockStoreWriter;
use crate::fs::reader::BlockStoreReader;
use crate::fs::index::{Index, BlockIndexInfo};
use crate::BlockIterator;

pub struct BlockStore {
    index: Index,
    writer: BlockStoreWriter,
    reader: BlockStoreReader,
    latest: Block,
    path: Arc<PathBuf>,
}

impl BlockStore {
    pub fn open(path: impl Into<PathBuf>) -> Result<BlockStore> {
        let path = Arc::new(path.into());
        let index_path = path.join("index");
        fs::create_dir_all(&index_path)?;
        let db = DB::open_default(
            index_path
                .to_str()
                .ok_or_else(|| "get path str error".to_string())?,
        )?;
        let index = Index::new(db);
        let cp = index.get_check_point()?;

        let blk_path = Arc::new(path.join("chain"));
        fs::create_dir_all(&*blk_path.clone())?;

        // TODO: process sync block form files into index when index was clear.
        // or. index check point belong block files, we must clean index and sync again.

        let writer = BlockStoreWriter::new(blk_path.clone(), cp)?;
        let reader = BlockStoreReader::new(blk_path);

        Ok(BlockStore {
            index,
            writer,
            reader,
            // TODO init
            latest: Block {
                header: None,
                data: None,
                metadata: None,
            },
            path,
        })
    }
}

impl crate::BlockStore for BlockStore {
    fn add_block(&mut self, block: &Block) -> Result<()> {
        self.latest = block.clone();
        let fp = self.writer.save(block)?.into();
        self.index.refresh(BlockIndexInfo { fp, block })
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

    fn retrieve_block_by_number(&self, block_num: u64) -> Result<Block> {
        let fp = self.index.get_fp_by_number(block_num)?;
        match fp {
            Some(fp) => self.reader.read_blk(fp),
            None => Ok(self.latest.clone()),
        }
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
}

#[cfg(test)]
mod tests {
    use crate::BlockStore;
    use silk_proto::*;
    use tempfile::TempDir;

    fn create_blk(i: u64) -> Block {
        let header = BlockHeader {
            number: i,
            previous_hash: format!("previous_hash_{:}", i).into_bytes(),
            data_hash: format!("data_hash_{:}", i).into_bytes(),
        };

        let data = BlockData {
            data: vec![format!("tx data: {:}", i).into_bytes()],
        };
        Block {
            header: Some(header),
            data: Some(data),
            metadata: None,
        }
    }
    #[test]
    fn test_store() {
        let temp_dir = TempDir::new().unwrap();
        let dir = temp_dir.path().to_str().unwrap();
        let mut store = super::BlockStore::open(dir).unwrap();

        for i in 0..1000 {
            store.add_block(&create_blk(i)).unwrap();
        }

        let b111 = store.retrieve_block_by_number(111).unwrap();
        assert_eq!(b111, create_blk(111))
    }
}
