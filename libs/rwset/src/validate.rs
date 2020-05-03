use error::*;

use silk_proto::*;
use statedb::{are_same, Height, VersionedDB};
use crate::builder::TxRwSet;

pub struct BlockValidate<V: VersionedDB>{
    vdb :V
}

impl <V: VersionedDB>BlockValidate<V> {
}


pub fn validate_writeset<V: VersionedDB>(tx_rw_set: &TxRwSet, vdb: V) -> Result<()> {
    for rw_set in &tx_rw_set.ns_rw_sets {
        let ns = rw_set.namespace.clone();

        //Validation of write set
        for kv_write in &rw_set.kv_rw_set.writes {
            vdb.validate_key_value(&kv_write.key, &kv_write.value)?;
        }
    }

    Ok(())
}


pub fn validate_tx<V: VersionedDB>(tx_rw_set: &TxRwSet, vdb: V) -> Result<TxValidationCode> {
    for rw_set in &tx_rw_set.ns_rw_sets {
        let ns = rw_set.namespace.clone();

        // Validation of read set
        for kv_read in &rw_set.kv_rw_set.reads {
            let committed_version = vdb.get_version(&ns, &kv_read.key)?;

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
                let committed_version = vdb.get_version(&ns, &key)?;

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
