use crate::TxRwSet;
use error::*;
use std::io;
use std::fs::File;

pub fn validate<F>(tx_rw_set: TxRwSet, f: F) -> Result<()>
    where
        F: Fn(String, &[u8]) -> Result<()>,
{
    for rw_set in tx_rw_set.ns_rw_sets {

        for kv_write in rw_set.kv_rw_set.writes {
            f(kv_write.key, &kv_write.value)?;
        }
    }
    Ok(())
}

