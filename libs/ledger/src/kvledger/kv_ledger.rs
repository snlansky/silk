use crate::kvledger::history;
use crate::simulator::TxSimulator;
use crate::QueryExecutor;
use error::*;
use silk_proto::*;

pub struct KVLedger {}

impl KVLedger {
    pub fn new() -> Self {
        KVLedger {}
    }
}

impl crate::Ledger for KVLedger {
    type HQE = history::KVHistoryQueryExecutor;

    fn get_blockchain_info(&self) -> Result<BlockchainInfo> {
        unimplemented!()
    }

    fn get_block_by_number(&self, _block_number: u64) -> Result<Block> {
        unimplemented!()
    }

    fn get_blocks_iterator(
        &self,
        _start_block_number: u64,
    ) -> Result<Box<dyn Iterator<Item = Block>>> {
        unimplemented!()
    }

    fn get_transaction_by_id(&self, _tx_id: String) -> Result<ProcessedTransaction> {
        unimplemented!()
    }

    fn get_block_by_hash(&self, _block_hash: Vec<u8>) -> Result<Block> {
        unimplemented!()
    }

    fn get_block_by_tx_id(&self, _tx_id: String) -> Result<Block> {
        unimplemented!()
    }

    fn get_tx_validation_code_by_tx_id(&self, _tx_id: String) -> Result<TxValidationCode> {
        unimplemented!()
    }

    fn new_tx_simulator(&self, _txid: String) -> Result<Box<dyn TxSimulator>> {
        unimplemented!()
    }

    fn new_query_executor(&self) -> Result<Box<dyn QueryExecutor>> {
        unimplemented!()
    }

    fn new_history_query_executor(&self) -> Result<Self::HQE> {
        unimplemented!()
    }

    fn commit_legacy(&self, _block: Block) -> Result<()> {
        unimplemented!()
    }

    fn close(&self) {
        unimplemented!()
    }
}
