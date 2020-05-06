use error::*;
use silk_proto::*;
use statedb::{Height, UpdateBatch, VersionedValue};
use std::collections::HashMap;
use crate::builder::TxRwSet;

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct CompositeKey {
    ns: String,
    coll: String,
    key: String,
}

type KeyOpsFlag = u8;

const UPSERT_VAL: KeyOpsFlag = 1;
const METADATA_UPDATE: KeyOpsFlag = 2;
const METADATA_DELETE: KeyOpsFlag = 4;
const KEY_DELETE: KeyOpsFlag = 8;

struct KeyOps {
    flag: KeyOpsFlag,
    value: Vec<u8>,
    metadata: Vec<u8>,
}

impl KeyOps {
    fn is_delete(&self) -> bool {
        self.flag & KEY_DELETE == KEY_DELETE
    }

    fn is_upsert_and_metadata_update(&self) -> bool {
        if self.flag & UPSERT_VAL == UPSERT_VAL {
            self.flag & METADATA_UPDATE == METADATA_UPDATE
                || self.flag & METADATA_DELETE == METADATA_DELETE
        } else {
            false
        }
    }

    fn is_only_upsert(&self) -> bool {
        self.flag | UPSERT_VAL == UPSERT_VAL
    }
}

#[derive(Default)]
pub struct TxOps {
    map: HashMap<CompositeKey, KeyOps>,
}

impl TxOps {
    fn upsert(&mut self, k: CompositeKey, val: Vec<u8>) {
        let key_ops = self.get_or_create_key_entry(k);
        key_ops.flag += UPSERT_VAL;
        key_ops.value = val;
    }

    fn delete(&mut self, k: CompositeKey) {
        let key_ops = self.get_or_create_key_entry(k);
        key_ops.flag += KEY_DELETE;
    }

    fn metadata_update(&mut self, k: CompositeKey, metadata: Vec<u8>) {
        let key_ops = self.get_or_create_key_entry(k);
        key_ops.flag += METADATA_UPDATE;
        key_ops.metadata = metadata;
    }

    fn metadata_delete(&mut self, k: CompositeKey) {
        let key_ops = self.get_or_create_key_entry(k);
        key_ops.flag += METADATA_DELETE;
    }

    fn get_or_create_key_entry(&mut self, k: CompositeKey) -> &mut KeyOps {
        if !self.map.contains_key(&k) {
            self.map.insert(
                k.clone(),
                KeyOps {
                    flag: 0,
                    value: vec![],
                    metadata: vec![],
                },
            );
        }
        self.map.get_mut(&k).unwrap()
    }

    pub fn apply_tx_rwset(&mut self, rwset: TxRwSet) -> Result<()> {
        for ns_reset in rwset.ns_rw_sets {
            let ns = ns_reset.namespace;

            for kv_write in ns_reset.kv_rw_set.writes {
                self.apply_kv_write(&ns, &String::default(), kv_write);
            }

            for kv_mate_write in ns_reset.kv_rw_set.metadata_writes {
                self.apply_metadata(&ns, &String::default(), kv_mate_write);
            }

            for coll_hash_rwset in ns_reset.coll_hashed_rw_sets {
                let coll = coll_hash_rwset.collection_name;

                for hashed_write in coll_hash_rwset.hashed_rw_set.hashed_writes {
                    self.apply_kv_write(
                        &ns,
                        &coll,
                        KvWrite {
                            key: String::from_utf8(hashed_write.key_hash)?,
                            is_delete: hashed_write.is_delete,
                            value: hashed_write.value_hash,
                        },
                    );
                }

                for metadata_write in coll_hash_rwset.hashed_rw_set.metadata_writes {
                    self.apply_metadata(
                        &ns,
                        &coll,
                        KvMetadataWrite {
                            key: String::from_utf8(metadata_write.key_hash)?,
                            entries: metadata_write.entries,
                        },
                    );
                }
            }
        }

        Ok(())
    }

    fn apply_kv_write(&mut self, ns: &String, coll: &String, kv_write: KvWrite) {
        let ck = CompositeKey {
            ns: ns.clone(),
            coll: coll.clone(),
            key: kv_write.key,
        };
        if kv_write.is_delete {
            self.delete(ck)
        } else {
            self.upsert(ck, kv_write.value)
        }
    }

    fn apply_metadata(
        &mut self,
        ns: &String,
        coll: &String,
        metadata_write: KvMetadataWrite,
    ) -> Result<()> {
        let ck = CompositeKey {
            ns: ns.clone(),
            coll: coll.clone(),
            key: metadata_write.key,
        };
        if metadata_write.entries.is_empty() {
            self.metadata_delete(ck);
        } else {
            let metadata = KvMetadataWrite {
                key: "".to_string(),
                entries: metadata_write.entries,
            };
            self.metadata_update(ck, utils::proto::marshal(&metadata)?);
        }
        Ok(())
    }
}

pub fn apply_write_set(tx_rwset: TxRwSet, height: Height) -> Result<UpdateBatch> {
    let mut txops = TxOps::default();
    txops.apply_tx_rwset(tx_rwset)?;

    let mut batch = UpdateBatch::new();

    for (CompositeKey { ns, coll, key }, key_ops) in txops.map {
        if coll.is_empty() {
            if key_ops.is_delete() {
                batch.delete(&ns, &key, height.clone());
            } else {
                batch.put_val_and_metadata(
                    &ns,
                    &key,
                    key_ops.value,
                    key_ops.metadata,
                    height.clone(),
                );
            }
        } else {
            // TODO
            error!("unimplemented!")
        }
    }

    Ok(batch)
}

pub struct  PubAndHashUpdates{
    pub PubUpdates: UpdateBatch,
    pub HashUpdates: HashMap<String, UpdateBatch> , // maintains entries of tuple <Namespace, UpdatesForNamespace>
}

impl PubAndHashUpdates {
    pub fn new() -> Self {
        PubAndHashUpdates{ PubUpdates: UpdateBatch::new(), HashUpdates: Default::default() }
    }

    pub fn apply_write_set(&mut self, tx_rw_set: TxRwSet, tx_height: Height) -> Result<()> {
        let mut tx_ops = TxOps::default();
        tx_ops.apply_tx_rwset(tx_rw_set)?;

        for (ck, key_ops) in tx_ops.map {
            let CompositeKey{ns, coll, key} = ck;
            if coll.eq("") {
                if key_ops.is_delete() {
                    self.PubUpdates.update(&ns, &key, VersionedValue{
                        value: vec![],
                        metadata: vec![],
                        version: tx_height.clone(),
                    });
                } else {
                    self.PubUpdates.put_val_and_metadata(&ns, &key, key_ops.value, key_ops.metadata, tx_height.clone());
                }
            } else {
                if key_ops.is_delete() {
                    if !self.HashUpdates.contains_key(&ns) {
                        self.HashUpdates.insert(ns.clone(), UpdateBatch::new());
                    }
                    let batch = self.HashUpdates.get_mut(&ns).unwrap();
                    batch.delete(&coll, &key, tx_height);
                } else {
                    if !self.HashUpdates.contains_key(&ns) {
                        self.HashUpdates.insert(ns.clone(), UpdateBatch::new());
                    }
                    let batch = self.HashUpdates.get_mut(&ns).unwrap();
                    batch.put_val_and_metadata(&coll, &key, key_ops.value, key_ops.metadata, tx_height);
                }
            }
        }

        Ok(())
    }
}
