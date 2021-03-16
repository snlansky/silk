use error::*;

use crate::rwset::builder::TxRwSet;
use crate::rwset::key::{self, PubAndHashUpdates};
use crate::statedb::{self, Height, UpdateBatch, VersionedDB};
use silk_proto::*;
use std::collections::HashMap;
use std::convert::TryFrom;

pub struct Validator<V: VersionedDB> {
    vdb: V,
}

impl<V: VersionedDB> Validator<V> {
    pub fn new(vdb: V) -> Self {
        Validator { vdb }
    }

    pub fn validate_and_prepare_batch(
        &self,
        block: Block,
    ) -> Result<(UpdateBatch, Height, HashMap<String, TxValidationCode>)> {
        if let (Some(header), Some(data)) = (block.header, block.data) {
            let mut txs_filter = HashMap::new();
            let mut updates = PubAndHashUpdates::new();

            for (index, proto_msg) in data.data.iter().enumerate() {
                let tx: Transaction = utils::proto::unmarshal(proto_msg)?;
                let proposal = tx
                    .signed_proposal
                    .ok_or_else(|| from_str("proposal is null"))?;
                let tx_header = utils::proto::unmarshal::<Proposal>(&proposal.proposal_bytes)?
                    .header
                    .ok_or_else(|| from_str("transaction header is null"))?;

                let resp = tx
                    .response
                    .get(0)
                    .ok_or_else(|| from_str("transaction proposal response list is null"))?;
                let payload: ProposalResponsePayload = utils::proto::unmarshal(&resp.payload)?;
                let tx_read_write_set: TxReadWriteSet = utils::proto::unmarshal(&payload.results)?;
                let tx_rw_set = TxRwSet::try_from(tx_read_write_set)?;
                if self.validate_writeset(&tx_rw_set).is_err() {
                    // TODO:record this transaction
                    txs_filter.insert(tx_header.tx_id.clone(), TxValidationCode::InvalidWriteset);
                    continue;
                }

                let validation_code = self.validate_tx(&tx_rw_set, &mut updates)?;

                if validation_code == TxValidationCode::Valid {
                    debug!("Block [{:?}] Transaction index [{:?}] TxId [{:?}] marked as valid by state validator.  [{:?}]", header.number, index, tx_header.tx_id, validation_code);
                    let _ = updates
                        .apply_write_set(tx_rw_set, Height::new(header.number, index as u64));
                } else {
                    warn!("Block [{:?}] Transaction index [{:?}] TxId [{:?}] marked as invalid by state validator. Reason code [{:?}]",
                          header.number, index, tx_header.tx_id, validation_code);
                }
                txs_filter.insert(tx_header.tx_id.clone(), validation_code);
            }

            return Ok((
                UpdateBatch::from(updates),
                Height::new(header.number, (data.data.len() - 1) as u64),
                txs_filter,
            ));
        }

        Err(from_str("block content is null"))
    }

    fn validate_writeset(&self, tx_rw_set: &TxRwSet) -> Result<()> {
        for rw_set in &tx_rw_set.ns_rw_sets {
            //Validation of write set
            for kv_write in &rw_set.kv_rw_set.writes {
                self.vdb
                    .validate_key_value(&kv_write.key, &kv_write.value)?;
            }
        }

        Ok(())
    }

    fn validate_tx(
        &self,
        tx_rw_set: &TxRwSet,
        updates: &mut PubAndHashUpdates,
    ) -> Result<TxValidationCode> {
        for rw_set in &tx_rw_set.ns_rw_sets {
            let ns = rw_set.namespace.clone();

            // Validation of read set
            for kv_read in &rw_set.kv_rw_set.reads {
                if updates.pub_updates.exists(&ns, &kv_read.key) {
                    return Ok(TxValidationCode::MvccReadConflict);
                }

                let committed_version = self.vdb.get_version(&ns, &kv_read.key)?;

                let ver = kv_read.version.clone().map(Height::from);
                debug!(
                    "comparing versions for key [{:?}]: committed version={:?} and read version={:?}",
                    kv_read.key.clone(),
                    committed_version.clone(),
                    ver
                );
                if !statedb::are_same(committed_version, ver) {
                    debug!("Version mismatch for key [{:?}:{:?}]. committed version = [{:?}], version in read set [{:?}]",
                           ns, kv_read.key, committed_version, kv_read.version);
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
                let coll = coll_hashed_rw_set.collection_name.clone();

                for kv_read_hash in &coll_hashed_rw_set.hashed_rw_set.hashed_reads {
                    let hash = utils::base64::encode(&kv_read_hash.key_hash);

                    if updates.hash_updates.contains_key(&ns) {
                        let ns_batch = updates.hash_updates.get(&ns).unwrap();
                        if ns_batch.exists(&coll, &hash) {
                            return Ok(TxValidationCode::MvccReadConflict);
                        }
                    }

                    let committed_version = self
                        .vdb
                        .get_version(&key::derive_hashed_data_ns(&ns, &coll), &hash)?;

                    let ver = kv_read_hash.version.clone().map(Height::from);
                    if !statedb::are_same(committed_version, ver) {
                        debug!("Version mismatch for key hash [{:?}:{:?}:{:?}]. committed version = [{:?}], version in hashed read set [{:?}]",
                               ns,
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
