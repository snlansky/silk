use crate::statedb::{Height, VersionedValue};
use std::collections::HashMap;

#[derive(Debug)]
struct NsUpdates {
    pub m: HashMap<String, VersionedValue>,
}
impl NsUpdates {
    pub fn new() -> Self {
        NsUpdates { m: HashMap::new() }
    }
}

// UpdateBatch encloses the details of multiple `updates`
#[derive(Debug)]
pub struct UpdateBatch {
    contains_post_order_writes: bool,
    updates: HashMap<String, NsUpdates>,
}

impl UpdateBatch {
    pub fn new() -> Self {
        UpdateBatch {
            contains_post_order_writes: false,
            updates: HashMap::new(),
        }
    }

    pub fn get(&self, ns: &String, key: &String) -> Option<VersionedValue> {
        self.updates
            .get(ns)
            .and_then(|ns| ns.m.get(key))
            .map(|vv| vv.clone())
    }

    pub fn put(&mut self, ns: &String, key: &String, value: Vec<u8>, version: Height) {
        self.put_val_and_metadata(ns, key, value, vec![], version);
    }

    // put_val_and_metadata adds a key with value and metadata
    // TODO introducing a new function to limit the refactoring. Later in a separate CR, the 'Put' function above should be removed
    pub fn put_val_and_metadata(
        &mut self,
        ns: &String,
        key: &String,
        value: Vec<u8>,
        metadata: Vec<u8>,
        version: Height,
    ) {
        self.update(
            ns,
            key,
            VersionedValue {
                value,
                metadata,
                version,
            },
        )
    }

    // update updates the batch with a latest entry for a namespace and a key
    pub fn update(&mut self, ns: &String, key: &String, vv: VersionedValue) {
        self.get_or_create_nsupdates(ns).m.insert(key.clone(), vv);
    }

    // delete deletes a Key and associated value
    pub fn delete(&mut self, ns: &String, key: &String, version: Height) {
        self.update(
            ns,
            key,
            VersionedValue {
                value: vec![],
                metadata: vec![],
                version,
            },
        )
    }

    // exists checks whether the given key exists in the batch
    pub fn exists(&self, ns: &String, key: &String) -> bool {
        match self.updates.get(ns) {
            Some(ns) => ns.m.contains_key(key),
            None => false,
        }
    }

    // get_updated_namespaces returns the names of the namespaces that are updated
    pub fn get_updated_namespaces(&self) -> Vec<String> {
        let keys = self.updates.keys();
        keys.map(|s| s.clone()).collect()
    }

    pub fn get_updates(&self, ns: &String) -> Option<HashMap<String, VersionedValue>> {
        self.updates.get(ns).map(|s| s.m.clone())
    }

    // merge merges another updates batch with this updates batch
    pub fn merge(&mut self, batch: UpdateBatch) {
        self.contains_post_order_writes =
            self.contains_post_order_writes || batch.contains_post_order_writes;
        for (ns, ns_updates) in batch.updates {
            for (key, vv) in ns_updates.m {
                self.update(&ns, &key, vv)
            }
        }
    }

    fn get_or_create_nsupdates(&mut self, ns: &String) -> &mut NsUpdates {
        if !self.updates.contains_key(ns) {
            self.updates.insert(ns.clone(), NsUpdates::new());
        }

        self.updates.get_mut(ns).unwrap()
    }
}
