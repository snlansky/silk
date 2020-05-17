mod statedb;
mod staterocksdb;
mod version;

use error::*;
use silk_proto::VersionedValueProto;
pub use statedb::*;
pub use staterocksdb::*;
pub use version::{are_same, Height};
use crate::rwset::key::CompositeKey;

// VersionedDBProvider provides an instance of an versioned DB
pub trait VersionedDBProvider {
    type V: VersionedDB;
    // get_db_handle returns a handle to a VersionedDB
    fn get_db_handle(&self, id: String) -> Self::V;
    // close closes all the VersionedDB instances and releases any resources held by VersionedDBProvider
    fn close(&self) {}
}

// VersionedDB lists methods that a db is supposed to implement
pub trait VersionedDB {
    type Iter: ResultsIterator<VersionedKV>;

    // get_state gets the value for given namespace and key. For a contract, the namespace corresponds to the contractId
    fn get_state(&self, namespace: &String, key: &String) -> Result<Option<VersionedValue>>;

    // get_version gets the version for given namespace and key. For a contract, the namespace corresponds to the contractId
    fn get_version(&self, namespace: &String, key: &String) -> Result<Option<Height>>;

    // get_state_multiple_keys gets the values for multiple keys in a single call
    fn get_state_multiple_keys(
        &self,
        namespace: &String,
        keys: Vec<String>,
    ) -> Result<Vec<VersionedValue>>;

    // get_state_range_scan_iterator returns an iterator that contains all the key-values between given key ranges.
    // start_key is inclusive
    // end_key is exclusive
    // The returned ResultsIterator contains results of type *VersionedKV
    fn get_state_range_scan_iterator(
        &self,
        namespace: &String,
        start_key: &String,
        end_key: &String,
    ) -> Result<Self::Iter>;

    // execute_query executes the given query and returns an iterator that contains results of type *VersionedKV.
    fn execute_query(&self, namespace: &String, query: &String)
        -> Result<Self::Iter>;

    // apply_updates applies the batch to the underlying db.
    // height is the height of the highest transaction in the Batch that
    // a state db implementation is expected to ues as a save point
    fn apply_updates(&self, batch: UpdateBatch, height: Option<Height>) -> Result<()>;

    // get_latest_save_point returns the height of the highest transaction upto which
    // the state db is consistent
    fn get_latest_save_point(&self) -> Result<Option<Height>>;

    // validate_key_value tests whether the key and value is supported by the db implementation.
    // For instance, leveldb supports any bytes for the key while the couchdb supports only valid utf-8 string
    // TODO make the function validate_key_value return a specific error say ErrInvalidKeyValue
    // However, as of now, the both implementations of this function (leveldb and couchdb) are deterministic in returing an error
    // i.e., an error is returned only if the key-value are found to be invalid for the underlying db
    fn validate_key_value(&self, key: &String, value: &[u8]) -> Result<()>;

    // bytes_key_supported returns true if the implementation (underlying db) supports the any bytes to be used as key.
    // For instance, leveldb supports any bytes for the key while the couchdb supports only valid utf-8 string
    fn bytes_key_supported(&self) -> bool;

    // open opens the db
    fn open(&self) -> Result<()>;

    // close closes the db
    fn close(&self);
}

// VersionedValue encloses value and corresponding version
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VersionedValue {
    pub value: Vec<u8>,
    pub metadata: Vec<u8>,
    pub version: Height,
}

impl VersionedValue {
    pub fn is_delete(&self) -> bool {
        self.value.is_empty()
    }

    // decode_value decodes the statedb value bytes
    pub fn decode_value(encoded_value: &[u8]) -> Result<VersionedValue> {
        let msg = utils::proto::unmarshal::<VersionedValueProto>(encoded_value)?;
        let ver = Height::new_from_bytes(&msg.version_bytes)?;
        Ok(VersionedValue {
            value: msg.value,
            metadata: msg.metadata,
            version: ver,
        })
    }

    // encodeValue encodes the value, version, and metadata
    pub fn encode_value(&self) -> Result<Vec<u8>> {
        let bytes = utils::proto::marshal(&VersionedValueProto {
            version_bytes: self.version.to_bytes(),
            value: self.value.clone(),
            metadata: self.metadata.clone(),
        })?;
        Ok(bytes)
    }
}

// ResultsIterator iterates over query results
pub trait ResultsIterator<T> {
    fn next(&self) -> Result<T>;
    fn close(&self);
}

// VersionedKV encloses key and corresponding VersionedValue
pub struct VersionedKV {
    pub composite_key :CompositeKey,
    pub versioned_value : VersionedValue,
}

#[cfg(test)]
mod tests {
    use crate::statedb::UpdateBatch;
    use crate::{Height, VersionedDB, VersionedDBProvider, VersionedValue};
    use tempfile::TempDir;

    struct Support<S: super::VersionedDBProvider> {
        s: S,
    }

    #[test]
    fn test_provider() {
        let temp_dir = TempDir::new().unwrap();
        let support = Support {
            s: super::staterocksdb::VersionedDBRocksProvider::new(
                temp_dir.path().to_str().unwrap(),
            ),
        };
        let vdb = support.s.get_db_handle("chain_id".to_string());

        let mut batch = UpdateBatch::new();
        batch.put(
            &"ns1".to_string(),
            &"k1".to_string(),
            "v1".as_bytes().to_vec(),
            Height::new(1, 0),
        );
        batch.put(
            &"ns1".to_string(),
            &"k2".to_string(),
            "v2".as_bytes().to_vec(),
            Height::new(1, 1),
        );
        batch.put(
            &"ns1".to_string(),
            &"k3".to_string(),
            "v3".as_bytes().to_vec(),
            Height::new(1, 2),
        );
        vdb.apply_updates(batch, Some(Height::new(1, 3))).unwrap();

        let v1 = vdb
            .get_state(&"ns1".to_string(), &"k1".to_string())
            .unwrap()
            .unwrap();
        assert_eq!(
            v1,
            VersionedValue {
                value: "v1".as_bytes().to_vec(),
                metadata: vec![],
                version: Height::new(1, 0),
            }
        );

        let check_point = vdb.get_latest_save_point().unwrap().unwrap();
        assert_eq!(check_point, Height::new(1, 3))
    }
}
