use crate::TxRwSet;
use error::*;
use std::io;
use std::fs::File;
use statedb::{VersionedDB, Height, are_same};

pub fn validate<V: VersionedDB>(tx_rw_set: TxRwSet, vdb: V) -> Result<()> {

    for rw_set in tx_rw_set.ns_rw_sets {

        //Validation of write set
        for kv_write in rw_set.kv_rw_set.writes {
            vdb.validate_key_value(kv_write.key, &kv_write.value)?;
        }

        // Validation of read set
        for kv_read in rw_set.kv_rw_set.reads {
            let committed_version = vdb.get_version(rw_set.namespace.clone(), kv_read.key)?;

            let ver =kv_read.version.map(Height::from)
            debug!("Comparing versions for key [{:?}]: committed version={:?} and read version={:?}",
                   kv_read.key, committed_version, ver);
            if !are_same(committed_version, ver) {
                
            }

        }
    }


    //
    Ok(())
}

