#[macro_use]
extern crate log;

use silk_proto::*;
use error::*;
use crate::statedb::ResultsIterator;
use crate::simulator::TxSimulator;

pub mod rwset;
pub mod simulator;
pub mod statedb;
pub mod txmgr;

// LedgerProvider provides handle to ledger instances
pub trait LedgerProvider {
    type L: Ledger;
    // create creates a new ledger with the given genesis block.
    // This function guarantees that the creation of ledger and committing the genesis block would an atomic action
    // The chain id retrieved from the genesis block is treated as a ledger id
    fn create(genesis_block: Block) -> Result<Self::L>;
    // open opens an already created ledger
    fn open(ledger_id: String) -> Result<Self::L>;
    // exists tells whether the ledger with given id exists
    fn exists(ledger_id: String) -> Result<bool>;
    // list lists the ids of the existing ledgers
    fn list() -> Result<Vec<String>>;
    // close closes the PeerLedgerProvider
    fn close();
}

// Ledger differs from the OrdererLedger in that Ledger locally maintain a bitmask
// that tells apart valid transactions from invalid ones
pub trait Ledger {
    type BlockIter: ResultsIterator<Block>;
    type HQE: HistoryQueryExecutor;

    // get_blockchain_info returns basic info about blockchain
    fn get_blockchain_info() -> Result<BlockchainInfo>;
    // get_block_by_number returns block at a given height
    // block_number of  math.MaxUint64 will return last block
    fn get_block_by_number(block_number: u64) -> Result<Block>;
    // get_blocks_iterator returns an iterator that starts from `start_block_number`(inclusive).
    // The iterator is a blocking iterator i.e., it blocks till the next block gets available in the ledger
    // ResultsIterator contains type BlockHolder
    fn get_blocks_iterator(start_block_number: u64) -> Result<Self::BlockIter>;
    // close closes the ledger
    fn close();
    // get_transaction_by_id retrieves a transaction by id
    fn get_transaction_by_id(tx_id: String) -> Result<ProcessedTransaction>;
    // get_block_by_hash returns a block given it's hash
    fn get_block_by_hash(block_hash: Vec<u8>) -> Result<Block>;
    // get_block_by_tx_id returns a block which contains a transaction
    fn get_block_by_tx_id(tx_id: String) -> Result<Block>;
    // get_tx_validation_code_by_tx_id returns reason code of transaction validation
    fn get_tx_validation_code_by_tx_id(tx_id: String) -> Result<TxValidationCode>;
    // new_tx_simulator gives handle to a transaction simulator.
    // A client can obtain more than one 'TxSimulator's for parallel execution.
    // Any snapshoting/synchronization should be performed at the implementation level if required
    fn new_tx_simulator(txid: String) -> Result<Box<dyn TxSimulator>>;
    // new_query_executor gives handle to a query executor.
    // A client can obtain more than one 'QueryExecutor's for parallel execution.
    // Any synchronization should be performed at the implementation level if required
    fn new_query_executor() -> Result<Box<dyn QueryExecutor>>;
    // new_history_query_executor gives handle to a history query executor.
    // A client can obtain more than one 'HistoryQueryExecutor's for parallel execution.
    // Any synchronization should be performed at the implementation level if required
    fn new_history_query_executor() -> Result<Self::HQE>;

    // commit_legacy commits the block and the corresponding pvt data in an atomic operation following the v14 validation/commit path
    // TODO: add a new Commit() path that replaces commit_legacy() for the validation refactor described in FAB-12221
    fn commit_legacy(block: Block) -> Result<()>;
}

// SimpleQueryExecutor encapsulates basic functions
pub trait QueryExecutor {}

// HistoryQueryExecutor executes the history queries
pub trait HistoryQueryExecutor {
    type Iter: ResultsIterator<KeyModification>;
    // get_history_for_key retrieves the history of values for a key.
    // The returned ResultsIterator contains results of type *KeyModification which is defined in fabric-protos/ledger/queryresult.
    fn get_history_for_key(namespace: String, key: String) -> Result<Self::Iter>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
