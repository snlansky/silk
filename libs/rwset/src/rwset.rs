use silk_proto::*;
use error::*;
use std::convert::TryFrom;

// TxRwSet acts as a proxy of 'rwset.TxReadWriteSet' proto message and helps constructing Read-write set specifically for KV data model
pub struct  TxRwSet {
    pub NsRwSets :Vec<NsRwSet>,
}

// NsRwSet encapsulates 'kvrwset.KVRWSet' proto message for a specific name space (chaincode)
pub struct NsRwSet {
    pub NameSpace:        String,
    pub KvRwSet          :KvrwSet ,
    pub CollHashedRwSets :Vec<CollHashedRwSet>,
}

// CollHashedRwSet encapsulates 'kvrwset.HashedRWSet' proto message for a specific collection
pub struct  CollHashedRwSet {
    pub CollectionName: String,
    pub HashedRwSet    :HashedRwSet,
    pub PvtRwSetHash   :Vec<u8>,
}

impl TryFrom<TxReadWriteSet> for TxRwSet {
    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

    fn try_from(value: TxReadWriteSet) -> Result<Self> {
        unimplemented!()
    }
}
impl TxRwSet {
    fn new(proto_msg:TxReadWriteSet) -> Result<Self> {
        let mut ns_rw_sets = Vec::with_capacity(proto_msg.ns_rwset.len());
        for msg in proto_msg.ns_rwset {
            let ns_rw_set = nsRwSetFromProtoMsg(msg)?;
            ns_rw_sets.push(ns_rw_set);
        }

        Ok(TxRwSet{ NsRwSets: ns_rw_sets })
    }
}

fn nsRwSetFromProtoMsg(msg: NsReadWriteSet) -> Result<NsRwSet> {
    let kv_rw_set = utils::proto::unmarshal::<KvrwSet>(&msg.rwset)?;
    let mut coll_rw_set_list = Vec::with_capacity(msg.collection_hashed_rwset.len());

    for protoMsg in msg.collection_hashed_rwset {
        let coll_rw_set = CollHashedRwSet{
            CollectionName: protoMsg.collection_name,
            HashedRwSet: utils::proto::unmarshal::<HashedRwSet>(&protoMsg.hashed_rwset)?,
            PvtRwSetHash: protoMsg.pvt_rwset_hash
        };
        coll_rw_set_list.push(coll_rw_set);
    };

    let ns_rw_set = NsRwSet{
        NameSpace: msg.namespace,
        KvRwSet: kv_rw_set,
        CollHashedRwSets: coll_rw_set_list,
    };

    Ok(ns_rw_set)
}
