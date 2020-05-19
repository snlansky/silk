use silk_proto::*;
use error::*;
use crate::simulator::TxSimulator;
use crate::{QueryExecutor, HistoryQueryExecutor};
use crate::kvledger::history::{HistoryQueryResultsIterator, KVHistoryQueryExecutor};
use crate::statedb::ResultsIterator;

pub struct KVLedger {

}

impl KVLedger {
    pub fn new() -> Self {
        KVLedger{}
    }
}

impl crate::Ledger for KVLedger {
    type BlockIter = BlockIter;
    type HQE = KVHistoryQueryExecutor;

    fn get_blockchain_info(&self) -> Result<BlockchainInfo> {
        unimplemented!()
    }

    fn get_block_by_number(&self, block_number: u64) -> Result<Block> {
        unimplemented!()
    }

    fn get_blocks_iterator(&self, start_block_number: u64) -> Result<Self::BlockIter> {
        unimplemented!()
    }

    fn get_transaction_by_id(&self, tx_id: String) -> Result<ProcessedTransaction> {
        unimplemented!()
    }

    fn get_block_by_hash(&self, block_hash: Vec<u8>) -> Result<Block> {
        unimplemented!()
    }

    fn get_block_by_tx_id(&self, tx_id: String) -> Result<Block> {
        unimplemented!()
    }

    fn get_tx_validation_code_by_tx_id(&self, tx_id: String) -> Result<TxValidationCode> {
        unimplemented!()
    }

    fn new_tx_simulator(&self, txid: String) -> Result<Box<dyn TxSimulator>> {
        unimplemented!()
    }

    fn new_query_executor(&self) -> Result<Box<dyn QueryExecutor>> {
        unimplemented!()
    }

    fn new_history_query_executor(&self) -> Result<Self::HQE> {
        unimplemented!()
    }

    fn commit_legacy(&self, block: Block) -> Result<()> {
        unimplemented!()
    }

    fn close(&self) {
        unimplemented!()
    }
}

pub struct BlockIter {}

impl  ResultsIterator<Block> for BlockIter {
    fn next(&self) -> Result<Block> {
        unimplemented!()
    }

    fn close(&self) {
        unimplemented!()
    }
}
