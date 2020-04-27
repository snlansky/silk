use error::*;
use std::collections::HashMap;
use statedb::*;
use rwset::*;

pub trait TxSimulator {
    // GetState gets the value for given namespace and key. For a chaincode, the namespace corresponds to the chaincodeId
    fn GetState(&mut self, namespace: String, key: String) -> Result<Vec<u8>>;

    // SetState sets the given value for the given namespace and key. For a chaincode, the namespace corresponds to the chaincodeId
    fn SetState(&mut self, namespace: String, key: String, value: Vec<u8>) -> Result<()>;

    // DeleteState deletes the given namespace and key
    fn DeleteState(&mut self, namespace: String, key: String) -> Result<()>;

    // SetMultipleKeys sets the values for multiple keys in a single call
    fn SetStateMultipleKeys(&mut self, namespace: String, kvs: HashMap<String, Vec<u8>>) -> Result<()>;

    // SetStateMetadata sets the metadata associated with an existing key-tuple <namespace, key>
    fn SetStateMetadata(&mut self, namespace: String, key: String, metadata: HashMap<String, Vec<u8>>) -> Result<()>;

    // DeleteStateMetadata deletes the metadata (if any) associated with an existing key-tuple <namespace, key>
    fn DeleteStateMetadata(&mut self, namespace: String, key: String) -> Result<()>;

    // ExecuteUpdate for supporting rich data model (see comments on QueryExecutor above)
    fn ExecuteUpdate(&mut self, query: String) -> Result<()>;

    // GetTxSimulationResults encapsulates the results of the transaction simulation.
    // This should contain enough detail for
    // - The update in the state that would be caused if the transaction is to be committed
    // - The environment in which the transaction is executed so as to be able to decide the validity of the environment
    //   (at a later time on a different peer) during committing the transactions
    // Different ledger implementation (or configurations of a single implementation) may want to represent the above two pieces
    // of information in different way in order to support different data-models or optimize the information representations.
    // Returned type 'TxSimulationResults' contains the simulation results for both the public data and the private data.
    // The public data simulation results are expected to be used as in V1 while the private data simulation results are expected
    // to be used by the gossip to disseminate this to the other endorsers (in phase-2 of sidedb)
    fn GetTxSimulationResults(&mut self, ) -> Result<TxSimulationResults>;

    // GetStateMetadata returns the metadata for given namespace and key
    fn GetStateMetadata(&mut self, namespace: String, key: String) -> Result<HashMap<String, Vec<u8>>>;

    // GetStateMultipleKeys gets the values for multiple keys in a single call
    fn GetStateMultipleKeys(&mut self, namespace: String, keys: Vec<String>) -> Result<Vec<Vec<u8>>>;

    // GetStateRangeScanIterator returns an iterator that contains all the key-values between given key ranges.
    // startKey is included in the results and endKey is excluded. An empty startKey refers to the first available key
    // and an empty endKey refers to the last available key. For scanning all the keys, both the startKey and the endKey
    // can be supplied as empty:Strings. However, a full scan should be used judiciously for performance reasons.
    // The returned ResultsIterator contains results of type *KV which is defined in fabric-protos/ledger/queryresult.
    fn GetStateRangeScanIterator(&mut self, namespace: String, startKey: String, endKey: String) -> Result<Box<dyn ResultsIterator>>;

    // ExecuteQuery executes the given query and returns an iterator that contains results of type specific to the underlying data store.
    // Only used for state databases that support query
    // For a chaincode, the namespace corresponds to the chaincodeId
    // The returned ResultsIterator contains results of type *KV which is defined in fabric-protos/ledger/queryresult.
    fn ExecuteQuery(&mut self, namespace: String, query: String) -> Result<Box<dyn ResultsIterator>>;


    // Done releases resources occupied by the QueryExecutor
    fn Done(&mut self);
}

