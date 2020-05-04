use error::*;

use silk_proto::*;
use statedb::{are_same, Height, VersionedDB, UpdateBatch};
use crate::builder::TxRwSet;
use std::collections::HashMap;
use std::convert::TryFrom;
use crate::key::{TxOps, CompositeKey, PubAndHashUpdates};



pub struct Validator<V: VersionedDB> {
    vdb :V,
}

impl <V: VersionedDB> Validator <V> {
    pub fn new(vdb: V) -> Self {
        Validator{vdb}
    }

    pub fn validate_and_prepare_batch(&self, block: Block) -> Result<PubAndHashUpdates> {
        if let (Some(header), Some(data), Some(metadata)) = (block.header, block.data, block.metadata) {
            let mut txs_filter = HashMap::new();
            let mut updates = PubAndHashUpdates::new();

            for (index, proto_msg) in data.data.iter().enumerate() {
                let tx:Transaction = utils::proto::unmarshal(&proto_msg)?;
                let resp = tx.response.get(0).ok_or(from_str("transaction proposal response list is null"))?;
                let payload: ProposalResponsePayload = utils::proto::unmarshal(&resp.payload)?;
                let tx_read_write_set: TxReadWriteSet = utils::proto::unmarshal(&payload.results)?;
                let tx_rw_set= TxRwSet::try_from(tx_read_write_set)?;
                if self.validate_writeset(&tx_rw_set).is_err() {
                    // TODO:record this transaction
                    txs_filter.insert(index, TxValidationCode::InvalidWriteset);
                    continue
                }


            }

        }

        Err(from_str("block content is null"))
    }

    fn validate_writeset(&self, tx_rw_set: &TxRwSet) -> Result<()> {
        for rw_set in &tx_rw_set.ns_rw_sets {
            let ns = rw_set.namespace.clone();

            //Validation of write set
            for kv_write in &rw_set.kv_rw_set.writes {
                self.vdb.validate_key_value(&kv_write.key, &kv_write.value)?;
            }
        }

        Ok(())
    }


    pub fn validate_tx(&self, tx_rw_set: &TxRwSet, updates: &mut PubAndHashUpdates) -> Result<TxValidationCode> {
        for rw_set in &tx_rw_set.ns_rw_sets {
            let ns = rw_set.namespace.clone();

            // Validation of read set
            for kv_read in &rw_set.kv_rw_set.reads {
                let committed_version = self.vdb.get_version(&ns, &kv_read.key)?;

                let ver = kv_read.version.clone().map(Height::from);
                debug!(
                    "comparing versions for key [{:?}]: committed version={:?} and read version={:?}",
                    kv_read.key.clone(),
                    committed_version.clone(),
                    ver
                );
                if !are_same(committed_version.clone(), ver) {
                    debug!("Version mismatch for key [{:?}:{:?}]. committed version = [{:?}], version in read set [{:?}]",
                           ns.clone(), kv_read.key, committed_version, kv_read.version);
                    return Ok(TxValidationCode::MvccReadConflict);
                }
            }

            // Validate range queries for phantom items
            for rgi in &rw_set.kv_rw_set.range_queries_info {
                debug!(
                    "validate range query: ns={:?}, RangeQueryInfo={:?}",
                    ns.clone(),
                    rgi
                )
                // TODO:
            }

            // Validate hashes for private reads
            for coll_hashed_rw_set in &rw_set.coll_hashed_rw_sets {
                for kv_read_hash in &coll_hashed_rw_set.hashed_rw_set.hashed_reads {
                    let hash = utils::base64::encode(&kv_read_hash.key_hash);
                    let key = format!(
                        "{:}{:}{:}",
                        ns.clone(),
                        coll_hashed_rw_set.collection_name,
                        hash.clone()
                    );
                    let committed_version = self.vdb.get_version(&ns, &key)?;

                    let ver = kv_read_hash.version.clone().map(Height::from);
                    if !are_same(committed_version.clone(), ver) {
                        debug!("Version mismatch for key hash [{:?}:{:?}:{:?}]. committed version = [{:?}], version in hashed read set [{:?}]",
                               ns.clone(),
                               coll_hashed_rw_set.collection_name,
                               hash,
                               committed_version,
                               kv_read_hash.version);
                        return Ok(TxValidationCode::MvccReadConflict);
                    }
                }
            }
        }

        Ok(TxValidationCode::Valid)
    }
}
