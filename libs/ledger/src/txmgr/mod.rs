use error::*;
use silk_proto::Block;
use crate::simulator::TxSimulator;
use crate::statedb::{Height, VersionedDB};

pub trait TxMgr {
    type T: TxSimulator;
    fn new_tx_simulator(txid: String) -> Result<Self::T>;
    fn validate_and_prepare(block: Block) -> Result<()>;
    fn get_last_savepoint() -> Result<Height>;
    fn should_recover(last_available_block: u64) -> Result<(bool, u64)>;
    fn commit() -> Result<()>;
}

pub struct UnlockedTxMgr<V: VersionedDB> {
    vdb: V,
}

#[cfg(test)]
mod tests {
    use error::*;
    use silk_proto::*;
    
    use tempfile::TempDir;
    use crate::statedb::{VersionedDBRocksProvider, VersionedValue, VersionedDB, Height, VersionedDBProvider};
    use crate::rwset::validate::Validator;
    use crate::simulator::sim::BasedTxSimulator;
    use crate::simulator::TxSimulator;

    #[test]
    fn it_works() {
        let temp_dir = TempDir::new().unwrap();
        let provider = VersionedDBRocksProvider::new(temp_dir.into_path());
        let vdb = provider.get_db_handle("chain_id".to_string());
        let validate = Validator::new(vdb.clone());

        let mut sim = BasedTxSimulator::new("tx1".to_string(), vdb.clone());
        sim.set_state(
            &"contract_name".to_string(),
            &"k1".to_string(),
            Vec::from("v1"),
        )
        .unwrap();
        let results = sim.get_tx_simulation_results().unwrap();
        let tx = create_tx(results.simulation_results, "tx1".to_string()).unwrap();
        let block = create_block(vec![tx], 1);

        let (batch, h, _tx_code) = validate.validate_and_prepare_batch(block).unwrap();
        println!("{:?} \n {:?}", batch, h);
        vdb.apply_updates(batch, Some(h)).unwrap();

        let v1 = vdb
            .get_state(&"contract_name".to_string(), &"k1".to_string())
            .unwrap();
        assert_eq!(
            v1,
            Some(VersionedValue {
                value: Vec::from("v1"),
                metadata: vec![],
                version: Height {
                    block_num: 1,
                    tx_num: 0
                }
            })
        )
    }

    fn create_block(txs: Vec<Transaction>, num: u64) -> Block {
        let data = txs
            .iter()
            .map(|t| utils::proto::marshal(t).unwrap())
            .collect();

        Block {
            header: Some(BlockHeader {
                number: num,
                previous_hash: vec![],
                data_hash: vec![],
            }),
            data: Some(BlockData { data }),
            metadata: None,
        }
    }
    fn create_tx(rw_set: TxReadWriteSet, txid: String) -> Result<Transaction> {
        let payload = ContractProposalPayload {
            contract_id: None,
            input: None,
            transient_map: Default::default(),
            timeout: 0,
        };

        let proposal = Proposal {
            header: Some(Header {
                header_type: HeaderType::Invoke as i32,
                version: 0,
                timestamp: None,
                channel_id: "chain_id".to_string(),
                tx_id: txid,
                tls_cert_hash: vec![],
                creator: vec![],
                nonce: vec![],
            }),
            payload: utils::proto::marshal(&payload)?,
        };
        let sp = SignedProposal {
            proposal_bytes: utils::proto::marshal(&proposal)?,
            signature: vec![],
        };

        let payload = ProposalResponsePayload {
            results: utils::proto::marshal(&rw_set)?,
            events: vec![],
        };
        let proposal_response = ProposalResponse {
            version: 0,
            timestamp: None,
            response: None,
            payload: utils::proto::marshal(&payload)?,
            endorsement: None,
        };

        let tx = Transaction {
            signed_proposal: Some(sp),
            response: vec![proposal_response],
        };
        Ok(tx)
    }

    #[test]
    fn test_mvcc() {
        let temp_dir = TempDir::new().unwrap();
        let provider = VersionedDBRocksProvider::new(temp_dir.into_path());
        let vdb = provider.get_db_handle("chain_id".to_string());
        let validate = Validator::new(vdb.clone());

        {
            let mut sim = BasedTxSimulator::new("tx0".to_string(), vdb.clone());
            sim.set_state(&"ns".to_string(), &"key1".to_string(), Vec::from("value1"))
                .unwrap();
            sim.set_state(&"ns".to_string(), &"key2".to_string(), Vec::from("value2"))
                .unwrap();
            sim.set_state(&"ns".to_string(), &"key3".to_string(), Vec::from("value3"))
                .unwrap();
            let results = sim.get_tx_simulation_results().unwrap();

            let tx = create_tx(results.simulation_results, "tx0".to_string()).unwrap();
            let block = create_block(vec![tx], 1);

            let (batch, h, _) = validate.validate_and_prepare_batch(block).unwrap();
            println!("{:?} \n {:?}", batch, h);
            vdb.apply_updates(batch, Some(h)).unwrap();
        }

        {
            let tx1 = {
                let mut sim = BasedTxSimulator::new("tx1".to_string(), vdb.clone());
                let val_key1 = sim
                    .get_state(&"ns".to_string(), &"key1".to_string())
                    .unwrap();
                assert_eq!(val_key1, Vec::from("value1"));
                sim.set_state(&"ns".to_string(), &"key1".to_string(), Vec::from("1"))
                    .unwrap();
                let results = sim.get_tx_simulation_results().unwrap();

                create_tx(results.simulation_results, "tx1".to_string()).unwrap()
            };
            let tx2 = {
                let mut sim = BasedTxSimulator::new("tx2".to_string(), vdb.clone());
                let _val_key1 = sim
                    .get_state(&"ns".to_string(), &"key1".to_string())
                    .unwrap();
                sim.set_state(&"ns".to_string(), &"key1".to_string(), Vec::from("2"))
                    .unwrap();
                let results = sim.get_tx_simulation_results().unwrap();

                create_tx(results.simulation_results, "tx2".to_string()).unwrap()
            };

            let block = create_block(vec![tx1, tx2], 2);

            let (batch, h, tx_code) = validate.validate_and_prepare_batch(block).unwrap();
            println!("{:?} \n {:?}\n {:?}", batch, h, tx_code);
            vdb.apply_updates(batch, Some(h)).unwrap();

            assert_eq!(tx_code.get("tx1"), Some(&TxValidationCode::Valid));
            assert_eq!(
                tx_code.get("tx2"),
                Some(&TxValidationCode::MvccReadConflict)
            );

            let v1 = vdb
                .get_state(&"ns".to_string(), &"key1".to_string())
                .unwrap();
            assert_eq!(
                v1,
                Some(VersionedValue {
                    value: Vec::from("1"),
                    metadata: vec![],
                    version: Height {
                        block_num: 2,
                        tx_num: 0
                    }
                })
            )

        }
    }
}
