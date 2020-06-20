use crate::rwset::builder::{RWSetBuilder, TxSimulationResults};
use crate::statedb::{Height, VersionedDB, VersionedValue};
use error::*;
use silk_proto::Kv;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;

pub struct BasedTxSimulator<V: VersionedDB> {
    tx_id: String,
    rw_set_builder: RWSetBuilder,
    vdb: V,
}
impl<V: VersionedDB> BasedTxSimulator<V> {
    pub fn new(tx_id: String, vdb: V) -> Self {
        BasedTxSimulator {
            tx_id,
            rw_set_builder: RWSetBuilder::new(),
            vdb,
        }
    }
}

impl<V: VersionedDB> super::TxSimulator for BasedTxSimulator<V> {
    fn get_state(&mut self, namespace: &str, key: &str) -> Result<Vec<u8>> {
        let v = self.vdb.get_state(namespace, key)?;
        let vv = v.unwrap_or(VersionedValue {
            value: vec![],
            metadata: vec![],
            version: Height {
                block_num: 0,
                tx_num: 0,
            },
        });
        self.rw_set_builder
            .add_to_read_set(namespace, key, vv.version);
        Ok(vv.value)
    }

    fn set_state(&mut self, namespace: &str, key: &str, value: Vec<u8>) -> Result<()> {
        self.rw_set_builder.add_to_write_set(namespace, key, value);
        Ok(())
    }

    fn delete_state(&mut self, namespace: &str, key: &str) -> Result<()> {
        self.set_state(namespace, key, vec![])
    }

    fn set_state_multiple_keys(
        &mut self,
        _namespace: &str,
        _kvs: HashMap<String, Vec<u8>, RandomState>,
    ) -> Result<()> {
        unimplemented!()
    }

    fn execute_update(&mut self, _query: &str) -> Result<()> {
        unimplemented!()
    }

    fn get_tx_simulation_results(&mut self) -> Result<TxSimulationResults> {
        self.rw_set_builder.get_tx_simulation_results()
    }

    fn get_state_metadata(
        &mut self,
        _namespace: &str,
        _key: &str,
    ) -> Result<HashMap<String, Vec<u8>, RandomState>> {
        unimplemented!()
    }

    fn get_state_multiple_keys(
        &mut self,
        _namespace: &str,
        _keys: Vec<String>,
    ) -> Result<Vec<Vec<u8>>> {
        unimplemented!()
    }

    fn get_state_range_scan_iterator(
        &mut self,
        _namespace: &str,
        _start_key: &str,
        _end_key: &str,
    ) -> Result<Box<dyn Iterator<Item = Kv>>> {
        unimplemented!()
    }

    fn execute_query(
        &mut self,
        _namespace: &str,
        _query: &str,
    ) -> Result<Box<dyn Iterator<Item = Kv>>> {
        unimplemented!()
    }

    fn done(&mut self) {
        unimplemented!()
    }
}
