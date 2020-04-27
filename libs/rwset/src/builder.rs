use std::collections::HashMap;
use silk_proto::*;
use error::*;
use statedb::Height;

// RWSetBuilder helps building the read-write set
pub struct RWSetBuilder {
    map: HashMap<String, NsRwBuilder>,
}

impl RWSetBuilder {
    pub fn new() -> Self {
        RWSetBuilder{map:HashMap::new()}
    }

    // add_to_read_set adds a key and corresponding version to the read-set
    fn add_to_read_set(&mut self, ns :String, key: String, version: Height) {
        let ns_rw_builder = self.get_or_create_ns_rw_builder(ns);
        let ver = Version{ block_num: version.block_num, tx_num: version.tx_num };
        ns_rw_builder.read_map.insert(key.clone(), KvRead{ key, version: Some(ver) });
    }

    // add_to_write_set adds a key and value to the write-set
    fn add_to_write_set(&mut self, ns :String, key: String, value: Vec<u8>) {
        let ns_rw_builder = self.get_or_create_ns_rw_builder(ns);

        ns_rw_builder.write_map.insert(key.clone(), KvWrite{
            key,
            is_delete: value.is_empty(),
            value
        });
    }

    // add_to_range_query_set adds a range query info for performing phantom read validation
    fn add_to_range_query_set(&mut self, ns: String, rqi:RangeQueryInfo) {
        let ns_rw_builder = self.get_or_create_ns_rw_builder(ns);
        let key = RangeQueryKey{
            start_key: rqi.start_key.clone(),
            end_key: rqi.end_key.clone(),
            itr_exhausted: rqi.itr_exhausted.clone()
        };
        if !ns_rw_builder.range_queries_map.contains_key(&key) {
            ns_rw_builder.range_queries_map.insert(key.clone(), rqi);
            ns_rw_builder.range_queries_keys.push(key);
        }
    }

    // get_tx_simulation_results returns the proto bytes of public rwset
    // (public data + hashes of private data) and the private rwset for the transaction
    fn get_tx_simulation_results(&self) -> Result<TxSimulationResults> {
        // Compute the proto bytes for pub rwset
        let rwset = self.get_tx_read_write_set();

        let sim = TxSimulationResults{
            pub_simulation_results: TxReadWriteSet { data_model: 0, ns_rwset: vec![] }
        };
        Ok(sim)
    }

    // get_tx_read_write_set returns the read-write set
    // TODO make this function private once txmgr starts using new function `get_tx_simulation_results` introduced here
    pub fn get_tx_read_write_set(&self) -> TxRwSet {
        unimplemented!()
    }

    fn get_or_create_ns_rw_builder(&mut self, ns: String) -> &mut NsRwBuilder {
        unimplemented!()
    }

    fn get_or_create_coll_hashed_rw_builder(&mut self, ns: String, coll: String) -> &mut CollHashRwBuilder {
        let ns_rw_builder = self.get_or_create_ns_rw_builder(ns);
        if !ns_rw_builder.coll_hash_rw_builder.contains_key(&coll) {
            ns_rw_builder.coll_hash_rw_builder.insert(coll.clone(), CollHashRwBuilder {
                coll_name: coll.clone(),
                read_map: Default::default(),
                write_map: Default::default(),
            });
        }

        ns_rw_builder.coll_hash_rw_builder.get_mut(&coll).unwrap()
    }
}

pub struct NsRwBuilder {
    namespace: String,
    read_map: HashMap<String, KvRead>, //for mvcc validation
    write_map: HashMap<String, KvWrite>,
    range_queries_map: HashMap<RangeQueryKey, RangeQueryInfo>,
    range_queries_keys: Vec<RangeQueryKey>,
    coll_hash_rw_builder: HashMap<String, CollHashRwBuilder>,
}

struct CollPvtRwBuilder {
    collection_name: String,
    write_map: HashMap<String, KvWrite>,
}

#[derive(Clone, Hash, Eq, PartialEq)]
struct RangeQueryKey {
    start_key: String,
    end_key: String,
    itr_exhausted: bool,
}

struct CollHashRwBuilder {
    coll_name: String,
    read_map: HashMap<String, KvReadHash>,
    write_map: HashMap<String, KvWriteHash>,
}

// TxSimulationResults captures the details of the simulation results
pub struct TxSimulationResults {
    pub simulation_results: TxReadWriteSet
}

// TxRwSet acts as a proxy of 'rwset.TxReadWriteSet' proto message and helps constructing Read-write set specifically for KV data model
pub struct TxRwSet {
    pub ns_rw_sets: Vec<NsRwSet>,
}

// NsRwSet encapsulates 'kvrwset.KVRWSet' proto message for a specific name space (chaincode)
pub struct NsRwSet {
    pub name_space: String,
    pub kv_rw_set: KvrwSet,
    pub coll_hashed_rw_sets: Vec<CollHashedRwSet>,
}

// CollHashedRwSet encapsulates 'kvrwset.HashedRWSet' proto message for a specific collection
pub struct CollHashedRwSet {
    pub collection_name: String,
    pub hashed_rw_set: HashedRwSet,
}
