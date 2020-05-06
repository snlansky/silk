use error::*;
use serde::export::TryFrom;
use silk_proto::tx_read_write_set::DataModel;
use silk_proto::*;
use statedb::Height;
use std::collections::HashMap;

// RWSetBuilder helps building the read-write set
pub struct RWSetBuilder {
    map: HashMap<String, NsRwBuilder>,
}

impl RWSetBuilder {
    pub fn new() -> Self {
        RWSetBuilder {
            map: HashMap::new(),
        }
    }

    // add_to_read_set adds a key and corresponding version to the read-set
    pub fn add_to_read_set(&mut self, ns: &String, key: &String, version: Height) {
        let ns_rw_builder = self.get_or_create_ns_rw_builder(ns);
        let ver = Version {
            block_num: version.block_num,
            tx_num: version.tx_num,
        };
        ns_rw_builder.read_map.insert(
            key.clone(),
            KvRead {
                key: key.clone(),
                version: Some(ver),
            },
        );
    }

    // add_to_write_set adds a key and value to the write-set
    pub fn add_to_write_set(&mut self, ns: &String, key: &String, value: Vec<u8>) {
        let ns_rw_builder = self.get_or_create_ns_rw_builder(ns);

        ns_rw_builder.write_map.insert(
            key.clone(),
            KvWrite {
                key: key.clone(),
                is_delete: value.is_empty(),
                value,
            },
        );
    }

    // add_to_range_query_set adds a range query info for performing phantom read validation
    pub fn add_to_range_query_set(&mut self, ns: &String, rqi: RangeQueryInfo) {
        let ns_rw_builder = self.get_or_create_ns_rw_builder(ns);
        let key = RangeQueryKey {
            start_key: rqi.start_key.clone(),
            end_key: rqi.end_key.clone(),
            itr_exhausted: rqi.itr_exhausted.clone(),
        };
        if !ns_rw_builder.range_queries_map.contains_key(&key) {
            ns_rw_builder.range_queries_map.insert(key.clone(), rqi);
            ns_rw_builder.range_queries_keys.push(key);
        }
    }

    // get_tx_simulation_results returns the proto bytes of public rwset
    // (public data + hashes of private data) and the private rwset for the transaction
    pub fn get_tx_simulation_results(&self) -> Result<TxSimulationResults> {
        // Compute the proto bytes for pub rwset
        let rwset = self.get_tx_read_write_set();

        let sim = TxSimulationResults {
            simulation_results: TxReadWriteSet::try_from(rwset)?,
        };
        Ok(sim)
    }

    // get_tx_read_write_set returns the read-write set
    // TODO make this function private once txmgr starts using new function `get_tx_simulation_results` introduced here
    pub fn get_tx_read_write_set(&self) -> TxRwSet {
        let builders = get_values_by_sorted_keys(&self.map);
        let mut ns_rw_sets = Vec::with_capacity(builders.len());
        for builder in builders {
            ns_rw_sets.push(NsRwSet::from(builder));
        }
        return TxRwSet { ns_rw_sets };
    }

    fn get_or_create_ns_rw_builder(&mut self, ns: &String) -> &mut NsRwBuilder {
        if !self.map.contains_key(ns) {
            self.map.insert(ns.clone(), NsRwBuilder::new(ns.clone()));
        }
        self.map.get_mut(ns).unwrap()
    }

    fn get_or_create_coll_hashed_rw_builder(
        &mut self,
        ns: &String,
        coll: &String,
    ) -> &mut CollHashRwBuilder {
        let ns_rw_builder = self.get_or_create_ns_rw_builder(ns);
        if !ns_rw_builder.coll_hash_rw_builder.contains_key(coll) {
            ns_rw_builder.coll_hash_rw_builder.insert(
                coll.clone(),
                CollHashRwBuilder {
                    coll_name: coll.clone(),
                    read_map: Default::default(),
                    write_map: Default::default(),
                },
            );
        }

        ns_rw_builder.coll_hash_rw_builder.get_mut(coll).unwrap()
    }
}

pub fn get_values_by_sorted_keys<T: Clone>(map: &HashMap<String, T>) -> Vec<T> {
    let mut keys = map.iter().map(|k| k.0.clone()).collect::<Vec<String>>();
    keys.sort();
    let mut v = Vec::with_capacity(map.len());
    for key in keys {
        v.push(map.get(&key).unwrap().clone());
    }
    v
}

#[derive(Clone)]
pub struct NsRwBuilder {
    namespace: String,
    read_map: HashMap<String, KvRead>, //for mvcc validation
    write_map: HashMap<String, KvWrite>,
    range_queries_map: HashMap<RangeQueryKey, RangeQueryInfo>,
    range_queries_keys: Vec<RangeQueryKey>,
    coll_hash_rw_builder: HashMap<String, CollHashRwBuilder>,
}

impl NsRwBuilder {
    fn new(namespace: String) -> Self {
        NsRwBuilder {
            namespace,
            read_map: Default::default(),
            write_map: Default::default(),
            range_queries_map: Default::default(),
            range_queries_keys: vec![],
            coll_hash_rw_builder: Default::default(),
        }
    }
}

impl From<NsRwBuilder> for NsRwSet {
    fn from(value: NsRwBuilder) -> Self {
        let read_set = get_values_by_sorted_keys(&value.read_map);
        let write_set = get_values_by_sorted_keys(&value.write_map);

        let range_queries_info = value
            .range_queries_keys
            .iter()
            .map(|key| value.range_queries_map.get(key).unwrap().clone())
            .collect::<Vec<RangeQueryInfo>>();

        let coll_builders = get_values_by_sorted_keys(&value.coll_hash_rw_builder);
        let coll_hashed_rw_sets = coll_builders
            .into_iter()
            .map(|builder| CollHashedRwSet::from(builder))
            .collect::<Vec<CollHashedRwSet>>();

        NsRwSet {
            namespace: value.namespace,
            kv_rw_set: KvrwSet {
                reads: read_set,
                range_queries_info,
                writes: write_set,
                metadata_writes: vec![],
            },
            coll_hashed_rw_sets,
        }
    }
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

#[derive(Clone)]
struct CollHashRwBuilder {
    coll_name: String,
    read_map: HashMap<String, KvReadHash>,
    write_map: HashMap<String, KvWriteHash>,
}

impl From<CollHashRwBuilder> for CollHashedRwSet {
    fn from(value: CollHashRwBuilder) -> Self {
        CollHashedRwSet {
            collection_name: value.coll_name,
            hashed_rw_set: HashedRwSet {
                hashed_reads: get_values_by_sorted_keys(&value.read_map),
                hashed_writes: get_values_by_sorted_keys(&value.write_map),
                metadata_writes: vec![],
            },
        }
    }
}

// TxSimulationResults captures the details of the simulation results
#[derive(Debug)]
pub struct TxSimulationResults {
    pub simulation_results: TxReadWriteSet,
}

// TxRwSet acts as a proxy of 'rwset.TxReadWriteSet' proto message and helps constructing Read-write set specifically for KV data model
#[derive(Clone, Debug)]
pub struct TxRwSet {
    pub ns_rw_sets: Vec<NsRwSet>,
}

impl TryFrom<TxRwSet> for TxReadWriteSet {
    type Error = Error;

    fn try_from(rwset: TxRwSet) -> Result<Self> {
        let mut ns_rwset = Vec::with_capacity(rwset.ns_rw_sets.len());
        for rws in rwset.ns_rw_sets {
            ns_rwset.push(NsReadWriteSet::try_from(rws)?);
        }
        Ok(TxReadWriteSet {
            data_model: DataModel::Kv as i32,
            ns_rwset,
        })
    }
}

// NsRwSet encapsulates 'kvrwset.KVRWSet' proto message for a specific name space (chaincode)
#[derive(Clone, Debug)]
pub struct NsRwSet {
    pub namespace: String,
    pub kv_rw_set: KvrwSet,
    pub coll_hashed_rw_sets: Vec<CollHashedRwSet>,
}

impl TryFrom<NsRwSet> for NsReadWriteSet {
    type Error = Error;

    fn try_from(value: NsRwSet) -> Result<Self> {
        let mut collection_hashed_rwset = Vec::with_capacity(value.coll_hashed_rw_sets.len());
        for rw_set in value.coll_hashed_rw_sets {
            collection_hashed_rwset.push(CollectionHashedReadWriteSet::try_from(rw_set)?)
        }

        Ok(NsReadWriteSet {
            namespace: value.namespace,
            rwset: utils::proto::marshal(&value.kv_rw_set)?,
            collection_hashed_rwset,
        })
    }
}

// CollHashedRwSet encapsulates 'kvrwset.HashedRWSet' proto message for a specific collection
#[derive(Clone, Debug)]
pub struct CollHashedRwSet {
    pub collection_name: String,
    pub hashed_rw_set: HashedRwSet,
}

impl TryFrom<CollHashedRwSet> for CollectionHashedReadWriteSet {
    type Error = Error;

    fn try_from(value: CollHashedRwSet) -> Result<Self> {
        Ok(CollectionHashedReadWriteSet {
            collection_name: value.collection_name,
            hashed_rwset: utils::proto::marshal(&value.hashed_rw_set)?,
            pvt_rwset_hash: vec![],
        })
    }
}

impl TryFrom<TxReadWriteSet> for TxRwSet {
    type Error = Error;

    fn try_from(value: TxReadWriteSet) -> Result<Self> {
        let mut ns_rw_sets = Vec::with_capacity(value.ns_rwset.len());
        for msg in value.ns_rwset {
            let ns_rw_set = NsRwSet::try_from(msg)?;
            ns_rw_sets.push(ns_rw_set);
        }

        Ok(TxRwSet { ns_rw_sets })
    }
}

impl TryFrom<NsReadWriteSet> for NsRwSet {
    type Error = Error;

    fn try_from(value: NsReadWriteSet) -> Result<Self> {
        let kv_rw_set = utils::proto::unmarshal::<KvrwSet>(&value.rwset)?;
        let mut coll_hashed_rw_sets = Vec::with_capacity(value.collection_hashed_rwset.len());

        for proto_msg in value.collection_hashed_rwset {
            let coll_rw_set = CollHashedRwSet {
                collection_name: proto_msg.collection_name,
                hashed_rw_set: utils::proto::unmarshal::<HashedRwSet>(&proto_msg.hashed_rwset)?,
            };
            coll_hashed_rw_sets.push(coll_rw_set);
        }

        let ns_rw_set = NsRwSet {
            namespace: value.namespace,
            kv_rw_set,
            coll_hashed_rw_sets,
        };

        Ok(ns_rw_set)
    }
}
