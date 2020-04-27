use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use error::*;
use rwset::TxSimulationResults;
use statedb::ResultsIterator;

pub struct TxSimulator {

}

impl super::TxSimulator for TxSimulator {
    fn get_state(&mut self, namespace: String, key: String) -> Result<Vec<u8>> {
        unimplemented!()
    }

    fn set_state(&mut self, namespace: String, key: String, value: Vec<u8>) -> Result<()> {
        unimplemented!()
    }

    fn delete_state(&mut self, namespace: String, key: String) -> Result<()> {
        unimplemented!()
    }

    fn set_state_multiple_keys(&mut self, namespace: String, kvs: HashMap<String, Vec<u8>, RandomState>) -> Result<()> {
        unimplemented!()
    }

    fn execute_update(&mut self, query: String) -> Result<()> {
        unimplemented!()
    }

    fn get_tx_simulation_results(&mut self) -> Result<TxSimulationResults> {
        unimplemented!()
    }

    fn get_state_metadata(&mut self, namespace: String, key: String) -> Result<HashMap<String, Vec<u8>, RandomState>> {
        unimplemented!()
    }

    fn get_state_multiple_keys(&mut self, namespace: String, keys: Vec<String>) -> Result<Vec<Vec<u8>>> {
        unimplemented!()
    }

    fn get_state_range_scan_iterator(&mut self, namespace: String, start_key: String, end_key: String) -> Result<Box<dyn ResultsIterator>> {
        unimplemented!()
    }

    fn execute_query(&mut self, namespace: String, query: String) -> Result<Box<dyn ResultsIterator>> {
        unimplemented!()
    }

    fn done(&mut self) {
        unimplemented!()
    }
}
