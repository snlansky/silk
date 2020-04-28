use silk_proto::*;
use error::*;
use std::convert::TryFrom;

// TxRwSet acts as a proxy of 'rwset.TxReadWriteSet' proto message and helps constructing Read-write set specifically for KV data model
pub struct  TxRwSet {
    pub ns_rw_sets:Vec<NsRwSet>,
}

// NsRwSet encapsulates 'kvrwset.KVRWSet' proto message for a specific name space (chaincode)
pub struct NsRwSet {
    pub namespace:        String,
    pub kv_rw_set:KvrwSet ,
    pub coll_hashed_rw_sets:Vec<CollHashedRwSet>,
}

// CollHashedRwSet encapsulates 'kvrwset.HashedRWSet' proto message for a specific collection
pub struct  CollHashedRwSet {
    pub collection_name: String,
    pub hashed_rw_set:HashedRwSet,
    pub pvt_rw_set_hash: Vec<u8>,
}

impl TryFrom<TxReadWriteSet> for TxRwSet {
    type Error = Error;

    fn try_from(value: TxReadWriteSet) -> Result<Self> {
        let mut ns_rw_sets = Vec::with_capacity(value.ns_rwset.len());
        for msg in value.ns_rwset {
            let ns_rw_set = NsRwSet::try_from(msg)?;
            ns_rw_sets.push(ns_rw_set);
        }

        Ok(TxRwSet{ ns_rw_sets })
    }
}

impl TryFrom<NsReadWriteSet> for NsRwSet {
    type Error = Error;

    fn try_from(value: NsReadWriteSet) -> Result<Self> {
        let kv_rw_set = utils::proto::unmarshal::<KvrwSet>(&value.rwset)?;
        let mut coll_hashed_rw_sets = Vec::with_capacity(value.collection_hashed_rwset.len());

        for protoMsg in value.collection_hashed_rwset {
            let coll_rw_set = CollHashedRwSet{
                collection_name: protoMsg.collection_name,
                hashed_rw_set: utils::proto::unmarshal::<HashedRwSet>(&protoMsg.hashed_rwset)?,
                pvt_rw_set_hash: protoMsg.pvt_rwset_hash
            };
            coll_hashed_rw_sets.push(coll_rw_set);
        };

        let ns_rw_set = NsRwSet{
            namespace: value.namespace,
            kv_rw_set,
            coll_hashed_rw_sets,
        };

        Ok(ns_rw_set)
    }
}
