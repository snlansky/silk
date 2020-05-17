use silk_proto::*;
use error::*;

#[macro_use]
extern crate log;

pub mod statedb;
pub mod simulator;
pub mod txmgr;
pub mod rwset;
// // LedgerProvider provides handle to ledger instances
// pub trait LedgerProvider {
//     // Create creates a new ledger with the given genesis block.
//     // This function guarantees that the creation of ledger and committing the genesis block would an atomic action
//     // The chain id retrieved from the genesis block is treated as a ledger id
//     fn Create(genesis_block: Block) -> Result<PeerLedger>;
//     // Open opens an already created ledger
//     fn Open(ledger_id: string) -> Result<PeerLedger>;
//     // Exists tells whether the ledger with given id exists
//     fn Exists(ledger_id: string) -> Result<bool>;
//     // List lists the ids of the existing ledgers
//     fn List() -> Result<Vec<string>>;
//     // Close closes the PeerLedgerProvider
//     fn Close();
// }
//
// // PeerLedger differs from the OrdererLedger in that PeerLedger locally maintain a bitmask
// // that tells apart valid transactions from invalid ones
// pub trait  PeerLedger {
//     // GetBlockchainInfo returns basic info about blockchain
//     fn GetBlockchainInfo() -> Result<BlockchainInfo>;
//     // GetBlockByNumber returns block at a given height
//     // blockNumber of  math.MaxUint64 will return last block
//     fn GetBlockByNumber(blockNumber: u64) -> Result<Block>;
//     // GetBlocksIterator returns an iterator that starts from `startBlockNumber`(inclusive).
//     // The iterator is a blocking iterator i.e., it blocks till the next block gets available in the ledger
//     // ResultsIterator contains type BlockHolder
//     fn GetBlocksIterator(startBlockNumber: u64) -> Result<ResultsIterator<Block>>;
//     // Close closes the ledger
//     fn Close();
//     // GetTransactionByID retrieves a transaction by id
//     fn GetTransactionByID(txID: String) -> Result<ProcessedTransaction>;
//     // GetBlockByHash returns a block given it's hash
//     fn GetBlockByHash(blockHash: Vec<u8>)  -> Result<Block>;
//     // GetBlockByTxID returns a block which contains a transaction
//     fn GetBlockByTxID(txID: String) -> Result<Block>;
//     // GetTxValidationCodeByTxID returns reason code of transaction validation
//     fn GetTxValidationCodeByTxID(txID: String) -> Result<TxValidationCode>;
//     // NewTxSimulator gives handle to a transaction simulator.
//     // A client can obtain more than one 'TxSimulator's for parallel execution.
//     // Any snapshoting/synchronization should be performed at the implementation level if required
//     fn NewTxSimulator(txid: String) -> Result<TxSimulator>;
//     // NewQueryExecutor gives handle to a query executor.
//     // A client can obtain more than one 'QueryExecutor's for parallel execution.
//     // Any synchronization should be performed at the implementation level if required
//     fn NewQueryExecutor() (QueryExecutor, error)
//     // NewHistoryQueryExecutor gives handle to a history query executor.
//     // A client can obtain more than one 'HistoryQueryExecutor's for parallel execution.
//     // Any synchronization should be performed at the implementation level if required
//     fn NewHistoryQueryExecutor() (HistoryQueryExecutor, error)
//
//     // CommitLegacy commits the block and the corresponding pvt data in an atomic operation following the v14 validation/commit path
//     // TODO: add a new Commit() path that replaces CommitLegacy() for the validation refactor described in FAB-12221
//     CommitLegacy(blockAndPvtdata *BlockAndPvtData, commitOpts *CommitOptions) error
//     // GetConfigHistoryRetriever returns the ConfigHistoryRetriever
//     GetConfigHistoryRetriever() (ConfigHistoryRetriever, error)
// }
//
// // SimpleQueryExecutor encapsulates basic functions
// type SimpleQueryExecutor interface {
// // GetState gets the value for given namespace and key. For a chaincode, the namespace corresponds to the chaincodeId
// GetState(namespace string, key string) ([]byte, error)
// // GetStateRangeScanIterator returns an iterator that contains all the key-values between given key ranges.
// // startKey is included in the results and endKey is excluded. An empty startKey refers to the first available key
// // and an empty endKey refers to the last available key. For scanning all the keys, both the startKey and the endKey
// // can be supplied as empty strings. However, a full scan should be used judiciously for performance reasons.
// // The returned ResultsIterator contains results of type *KV which is defined in fabric-protos/ledger/queryresult.
// GetStateRangeScanIterator(namespace string, startKey string, endKey string) (commonledger.ResultsIterator, error)
// // GetPrivateDataHash gets the hash of the value of a private data item identified by a tuple <namespace, collection, key>
// // Function `GetPrivateData` is only meaningful when it is invoked on a peer that is authorized to have the private data
// // for the collection <namespace, collection>. However, the function `GetPrivateDataHash` can be invoked on any peer
// // to get the hash of the current value
// GetPrivateDataHash(namespace, collection, key string) ([]byte, error)
// }
//
// // ResultsIterator - an iterator for query result set
// pub trait ResultsIterator<T>  {
//     // Next returns the next item in the result set. The `QueryResult` is expected to be nil when
//     // the iterator gets exhausted
//     fn Next() -> Result<T>;
//     // Close releases resources occupied by the iterator
//     fn Close();
// }

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
