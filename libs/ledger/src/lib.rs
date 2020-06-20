#[macro_use]
extern crate log;

use crate::simulator::TxSimulator;
use error::*;
use silk_proto::*;

pub mod kvledger;
pub mod ledger_mgmt;
pub mod rwset;
pub mod simulator;
pub mod statedb;
pub mod txmgr;

// Initializer encapsulates dependencies for LedgerProvider
pub struct Initializer {
    // root_fs_path is the top-level directory where ledger files are stored.
    pub root_fs_path: String,
}

// LedgerProvider provides handle to ledger instances
pub trait LedgerProvider {
    type L: Ledger;
    // create creates a new ledger with the given genesis block.
    // This function guarantees that the creation of ledger and committing the genesis block would an atomic action
    // The chain id retrieved from the genesis block is treated as a ledger id
    fn create(&self, genesis_block: &Block) -> Result<Self::L>;
    // open opens an already created ledger
    fn open(&self, ledger_id: String) -> Result<Self::L>;
    // exists tells whether the ledger with given id exists
    fn exists(&self, ledger_id: String) -> Result<bool>;
    // list lists the ids of the existing ledgers
    fn list(&self) -> Result<Vec<String>>;
    // close closes the PeerLedgerProvider
    fn close(&self);
}

// Ledger differs from the OrdererLedger in that Ledger locally maintain a bitmask
// that tells apart valid transactions from invalid ones
pub trait Ledger {
    type HQE: HistoryQueryExecutor;

    // get_blockchain_info returns basic info about blockchain
    fn get_blockchain_info(&self) -> Result<BlockchainInfo>;
    // get_block_by_number returns block at a given height
    // block_number of  math.MaxUint64 will return last block
    fn get_block_by_number(&self, block_number: u64) -> Result<Block>;
    // get_blocks_iterator returns an iterator that starts from `start_block_number`(inclusive).
    // The iterator is a blocking iterator i.e., it blocks till the next block gets available in the ledger
    // ResultsIterator contains type BlockHolder
    fn get_blocks_iterator(&self, start_block_number: u64) -> Result<Box<dyn Iterator<Item=Block>>>;
    // get_transaction_by_id retrieves a transaction by id
    fn get_transaction_by_id(&self, tx_id: String) -> Result<ProcessedTransaction>;
    // get_block_by_hash returns a block given it's hash
    fn get_block_by_hash(&self, block_hash: Vec<u8>) -> Result<Block>;
    // get_block_by_tx_id returns a block which contains a transaction
    fn get_block_by_tx_id(&self, tx_id: String) -> Result<Block>;
    // get_tx_validation_code_by_tx_id returns reason code of transaction validation
    fn get_tx_validation_code_by_tx_id(&self, tx_id: String) -> Result<TxValidationCode>;
    // new_tx_simulator gives handle to a transaction simulator.
    // A client can obtain more than one 'TxSimulator's for parallel execution.
    // Any snapshoting/synchronization should be performed at the implementation level if required
    fn new_tx_simulator(&self, txid: String) -> Result<Box<dyn TxSimulator>>;
    // new_query_executor gives handle to a query executor.
    // A client can obtain more than one 'QueryExecutor's for parallel execution.
    // Any synchronization should be performed at the implementation level if required
    fn new_query_executor(&self) -> Result<Box<dyn QueryExecutor>>;
    // new_history_query_executor gives handle to a history query executor.
    // A client can obtain more than one 'HistoryQueryExecutor's for parallel execution.
    // Any synchronization should be performed at the implementation level if required
    fn new_history_query_executor(&self) -> Result<Self::HQE>;
    // commit_legacy commits the block and the corresponding pvt data in an atomic operation following the v14 validation/commit path
    // TODO: add a new Commit() path that replaces commit_legacy() for the validation refactor described in FAB-12221
    fn commit_legacy(&self, block: Block) -> Result<()>;
    // close closes the ledger
    fn close(&self);
}

// SimpleQueryExecutor encapsulates basic functions
pub trait QueryExecutor {}

// HistoryQueryExecutor executes the history queries
pub trait HistoryQueryExecutor {
    // get_history_for_key retrieves the history of values for a key.
    // The returned ResultsIterator contains results of type *KeyModification which is defined in fabric-protos/ledger/queryresult.
    fn get_history_for_key(namespace: String, key: String) -> Result<Box<dyn Iterator<Item=KeyModification>>>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
