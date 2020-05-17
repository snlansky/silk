use error::*;
use std::collections::HashMap;
use crate::rwset::builder::TxSimulationResults;
use crate::statedb::ResultsIterator;
use silk_proto::Kv;

mod sim;

pub trait TxSimulator {
    // get_state gets the value for given namespace and key. For a chaincode, the namespace corresponds to the chaincodeId
    fn get_state(&mut self, namespace: &String, key: &String) -> Result<Vec<u8>>;

    // set_state sets the given value for the given namespace and key. For a chaincode, the namespace corresponds to the chaincodeId
    fn set_state(&mut self, namespace: &String, key: &String, value: Vec<u8>) -> Result<()>;

    // delete_state deletes the given namespace and key
    fn delete_state(&mut self, namespace: &String, key: &String) -> Result<()>;

    // SetMultipleKeys sets the values for multiple keys in a single call
    fn set_state_multiple_keys(
        &mut self,
        namespace: &String,
        kvs: HashMap<String, Vec<u8>>,
    ) -> Result<()>;

    // execute_update for supporting rich data model (see comments on QueryExecutor above)
    fn execute_update(&mut self, query: &String) -> Result<()>;

    // get_tx_simulation_results encapsulates the results of the transaction simulation.
    // This should contain enough detail for
    // - The update in the state that would be caused if the transaction is to be committed
    // - The environment in which the transaction is executed so as to be able to decide the validity of the environment
    //   (at a later time on a different peer) during committing the transactions
    // Different ledger implementation (or configurations of a single implementation) may want to represent the above two pieces
    // of information in different way in order to support different data-models or optimize the information representations.
    // Returned type 'TxSimulationResults' contains the simulation results for both the public data and the private data.
    // The public data simulation results are expected to be used as in V1 while the private data simulation results are expected
    // to be used by the gossip to disseminate this to the other endorsers (in phase-2 of sidedb)
    fn get_tx_simulation_results(&mut self) -> Result<TxSimulationResults>;

    // get_state_metadata returns the metadata for given namespace and key
    fn get_state_metadata(
        &mut self,
        namespace: &String,
        key: &String,
    ) -> Result<HashMap<String, Vec<u8>>>;

    // get_state_multiple_keys gets the values for multiple keys in a single call
    fn get_state_multiple_keys(
        &mut self,
        namespace: &String,
        keys: Vec<String>,
    ) -> Result<Vec<Vec<u8>>>;

    // get_state_range_scan_iterator returns an iterator that contains all the key-values between given key ranges.
    // start_key is included in the results and end_key is excluded. An empty start_key refers to the first available key
    // and an empty end_key refers to the last available key. For scanning all the keys, both the start_key and the end_key
    // can be supplied as empty:Strings. However, a full scan should be used judiciously for performance reasons.
    // The returned ResultsIterator contains results of type *KV which is defined in fabric-protos/ledger/queryresult.
    fn get_state_range_scan_iterator(
        &mut self,
        namespace: &String,
        start_key: &String,
        end_key: &String,
    ) -> Result<QueryResultsItr>;

    // execute_query executes the given query and returns an iterator that contains results of type specific to the underlying data store.
    // Only used for state databases that support query
    // For a chaincode, the namespace corresponds to the chaincodeId
    // The returned ResultsIterator contains results of type *KV which is defined in fabric-protos/ledger/queryresult.
    fn execute_query(
        &mut self,
        namespace: &String,
        query: &String,
    ) -> Result<QueryResultsItr>;

    // done releases resources occupied by the QueryExecutor
    fn done(&mut self);
}

pub struct QueryResultsItr {

}

impl ResultsIterator<Kv> for QueryResultsItr {
    fn next(&self) -> Result<Kv> {
        unimplemented!()
    }

    fn close(&self) {
        unimplemented!()
    }
}
