mod fs;
mod keys;
mod store;
use error::*;
use silk_proto::*;

// BlockStore - an interface for persisting and retrieving blocks
// An implementation of this interface is expected to take an argument
// of type `IndexConfig` which configures the block store on what items should be indexed
pub trait BlockStore {
    fn add_block(&mut self, block: &Block) -> Result<()>;
    fn get_blockchain_info(&self) -> Result<BlockchainInfo>;
    fn retrieve_blocks(
        &self,
        start_num: u64,
    ) -> Result<Box<dyn Iterator<Item = Result<Option<Block>>>>>;
    fn retrieve_block_by_hash(&self, block_hash: &[u8]) -> Result<Option<Block>>;
    fn retrieve_block_by_number(&self, block_num: u64) -> Result<Option<Block>>; // blockNum of math.MaxUint64 will return last block
    fn retrieve_tx_by_id(&self, tx_id: &str) -> Result<Option<Transaction>>;
    fn retrieve_tx_by_blocknum_txnum(
        &self,
        block_num: u64,
        tx_num: u64,
    ) -> Result<Option<Transaction>>;
    fn retrieve_block_by_txid(&self, tx_id: &str) -> Result<Option<Block>>;
    fn retrieve_tx_validationcode_by_txid(&self, tx_id: String) -> Result<TxValidationCode>;
    fn shutdown() {}
}

#[cfg(test)]
mod tests {

    #[test]
    fn test() {}
}
