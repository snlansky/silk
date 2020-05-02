use super::statedb::*;
use super::version::*;
use dashmap::DashMap;
use error::*;
use rocksdb::{WriteBatch, DB};

use super::*;
use std::iter::Iterator;
use std::path::PathBuf;
use std::sync::Arc;

const DATA_KEY_PREFIX: char = 'd';
const NS_KEY_SEP: u8 = 0x00;
const LAST_KEY_INDICATOR: u8 = 0x01;
const SAVE_POINT_KEY: char = 's';

pub struct VersionedDBRocksProvider {
    db: Arc<DB>,
    handler: DashMap<String, RocksDBVersion>,
}

impl VersionedDBRocksProvider {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let db = DB::open_default(path.into()).unwrap();

        VersionedDBRocksProvider {
            db: Arc::new(db),
            handler: DashMap::new(),
        }
    }
}

impl VersionedDBProvider for VersionedDBRocksProvider {
    type V = RocksDBVersion;

    fn get_db_handle(&self, id: String) -> RocksDBVersion {
        if !self.handler.contains_key(&id) {
            self.handler.insert(
                id.clone(),
                RocksDBVersion {
                    db: self.db.clone(),
                    name: id.clone(),
                },
            );
        }

        let db = self.handler.get(&id).unwrap();
        let db = &*db;
        db.clone()
    }

    fn close(&self) {}
}

#[derive(Clone)]
pub struct RocksDBVersion {
    db: Arc<DB>,
    name: String,
}

impl VersionedDB for RocksDBVersion {
    fn get_state(&self, namespace: &String, key: &String) -> Result<Option<VersionedValue>> {
        debug!("get_state(). ns={:}, key={:}", namespace, key);
        let db_val = self.db.get(encode_data_key(namespace, key))?;
        if db_val.is_none() {
            return Ok(None);
        }
        let db_val = db_val.unwrap();
        if db_val.is_empty() {
            return Ok(None);
        }

        VersionedValue::decode_value(&db_val).map(Some)
    }

    fn get_version(&self, namespace: &String, key: &String) -> Result<Option<Height>> {
        let v = self.get_state(namespace, key)?;
        let h = v.map(|v| v.version);
        Ok(h)
    }

    fn get_state_multiple_keys(
        &self,
        _namespace: &String,
        _keys: Vec<String>,
    ) -> Result<Vec<VersionedValue>> {
        unimplemented!()
    }

    fn get_state_range_scan_iterator(
        &self,
        _namespace: &String,
        _start_key: &String,
        _end_key: &String,
    ) -> Result<Box<dyn ResultsIterator>> {
        unimplemented!()
    }

    fn execute_query(
        &self,
        _namespace: &String,
        _query: &String,
    ) -> Result<Box<dyn ResultsIterator>> {
        unimplemented!()
    }

    fn apply_updates(&self, batch: UpdateBatch, height: Option<Height>) -> Result<()> {
        let mut db_batch = WriteBatch::default();

        for ns in batch.get_updated_namespaces() {
            let updates = batch.get_updates(ns.clone());
            if updates.is_some() {
                for (k, vv) in updates.unwrap() {
                    let data_key = encode_data_key(&ns, &k);
                    debug!(
                        "Channel [{}]: Applying key(string)=[{}] key(bytes)=[{:?}]",
                        self.name, k, data_key
                    );

                    if vv.value.is_empty() {
                        db_batch.delete(data_key);
                    } else {
                        db_batch.put(data_key, vv.encode_value()?);
                    }
                }
            }
        }

        height.map(|h| {
            db_batch.put(vec![SAVE_POINT_KEY as u8], h.to_bytes());
        });

        self.db.write(db_batch)?;
        Ok(())
    }

    fn get_latest_save_point(&self) -> Result<Option<Height>> {
        let bytes = self.db.get(vec![SAVE_POINT_KEY as u8])?;
        if bytes.is_none() {
            return Ok(None);
        }
        let bytes = bytes.unwrap();
        if bytes.is_empty() {
            return Ok(None);
        }

        let h = Height::new_from_bytes(&bytes)?;
        Ok(Some(h))
    }

    fn validate_key_value(&self, _key: &String, _value: &[u8]) -> Result<()> {
        // TODO
        Ok(())
    }


    fn bytes_key_supported(&self) -> bool {
        unimplemented!()
    }

    fn open(&self) -> Result<()> {
        unimplemented!()
    }

    fn close(&self) {
        unimplemented!()
    }
}

fn encode_data_key(ns: &String, key: &String) -> Vec<u8> {
    let mut v: Vec<u8> = Vec::new();
    v.push(DATA_KEY_PREFIX as u8);
    unsafe {
        v.append(ns.clone().as_mut_vec());
        v.push(NS_KEY_SEP);
        v.append(key.clone().as_mut_vec());
    }
    v
}

fn decode_data_key(encoded_data_key: Vec<u8>) -> (String, String) {
    let mut find = false;
    let mut ns = vec![];
    let mut key = vec![];
    for (index, c) in encoded_data_key.iter().enumerate() {
        if index.eq(&0) {
            continue;
        }
        if c.eq(&NS_KEY_SEP) {
            find = true;
            continue;
        }
        if !find {
            ns.push(c.clone())
        } else {
            key.push(c.clone())
        }
    }

    (
        String::from_utf8(ns).unwrap(),
        String::from_utf8(key).unwrap(),
    )
}

#[cfg(test)]
mod tests {
    use crate::staterocksdb::{decode_data_key, encode_data_key};

    #[test]
    fn test_key() {
        let encode_key = encode_data_key(&"mychain".to_string(), &"kvdb".to_string());
        let (ns, key) = decode_data_key(encode_key);
        assert_eq!(ns, "mychain".to_string());
        assert_eq!(key, "kvdb".to_string());
    }
}
