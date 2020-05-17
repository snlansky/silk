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
    // Create creates a new ledger with the given genesis block.
    // This function guarantees that the creation of ledger and committing the genesis block would an atomic action
    // The chain id retrieved from the genesis block is treated as a ledger id
    fn Create(genesis_block: Block) -> Result<Self::L>;
    // Open opens an already created ledger
    fn Open(ledger_id: String) -> Result<Self::L>;
    // Exists tells whether the ledger with given id exists
    fn Exists(ledger_id: String) -> Result<bool>;
    // List lists the ids of the existing ledgers
    fn List() -> Result<Vec<String>>;
    // Close closes the PeerLedgerProvider
    fn Close();
}

// Ledger differs from the OrdererLedger in that Ledger locally maintain a bitmask
// that tells apart valid transactions from invalid ones
pub trait Ledger {
    type BlockIter: ResultsIterator<Block>;
    type HQE: HistoryQueryExecutor;

    // GetBlockchainInfo returns basic info about blockchain
    fn GetBlockchainInfo() -> Result<BlockchainInfo>;
    // GetBlockByNumber returns block at a given height
    // blockNumber of  math.MaxUint64 will return last block
    fn GetBlockByNumber(blockNumber: u64) -> Result<Block>;
    // GetBlocksIterator returns an iterator that starts from `startBlockNumber`(inclusive).
    // The iterator is a blocking iterator i.e., it blocks till the next block gets available in the ledger
    // ResultsIterator contains type BlockHolder
    fn GetBlocksIterator(startBlockNumber: u64) -> Result<Self::BlockIter>;
    // Close closes the ledger
    fn Close();
    // GetTransactionByID retrieves a transaction by id
    fn GetTransactionByID(txID: String) -> Result<ProcessedTransaction>;
    // GetBlockByHash returns a block given it's hash
    fn GetBlockByHash(blockHash: Vec<u8>) -> Result<Block>;
    // GetBlockByTxID returns a block which contains a transaction
    fn GetBlockByTxID(txID: String) -> Result<Block>;
    // GetTxValidationCodeByTxID returns reason code of transaction validation
    fn GetTxValidationCodeByTxID(txID: String) -> Result<TxValidationCode>;
    // NewTxSimulator gives handle to a transaction simulator.
    // A client can obtain more than one 'TxSimulator's for parallel execution.
    // Any snapshoting/synchronization should be performed at the implementation level if required
    fn NewTxSimulator(txid: String) -> Result<Box<dyn TxSimulator>>;
    // NewQueryExecutor gives handle to a query executor.
    // A client can obtain more than one 'QueryExecutor's for parallel execution.
    // Any synchronization should be performed at the implementation level if required
    fn NewQueryExecutor() -> Result<Box<dyn QueryExecutor>>;
    // NewHistoryQueryExecutor gives handle to a history query executor.
    // A client can obtain more than one 'HistoryQueryExecutor's for parallel execution.
    // Any synchronization should be performed at the implementation level if required
    fn NewHistoryQueryExecutor() -> Result<Self::HQE>;

    // CommitLegacy commits the block and the corresponding pvt data in an atomic operation following the v14 validation/commit path
    // TODO: add a new Commit() path that replaces CommitLegacy() for the validation refactor described in FAB-12221
    fn CommitLegacy(block: Block) -> Result<()>;
}

// SimpleQueryExecutor encapsulates basic functions
pub trait QueryExecutor {}

// HistoryQueryExecutor executes the history queries
pub trait HistoryQueryExecutor {
    type Iter: ResultsIterator<KeyModification>;
    // GetHistoryForKey retrieves the history of values for a key.
// The returned ResultsIterator contains results of type *KeyModification which is defined in fabric-protos/ledger/queryresult.
    fn GetHistoryForKey(namespace: String, key: String) -> Result<Self::Iter>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
