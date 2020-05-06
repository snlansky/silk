use error::*;
use simulator::TxSimulator;
use silk_proto::Block;
use statedb::{Height, VersionedDB};

pub trait TxMgr {
    type T :TxSimulator;
    fn new_tx_simulator(txid: String) -> Result<Self::T>;
    fn validate_and_prepare(block: Block) -> Result<()>;
    fn get_last_savepoint() -> Result<Height>;
    fn should_recover(last_available_block:u64) -> Result<(bool, u64)>;
    fn commit() -> Result<()> ;
}

pub struct UnlockedTxMgr<V: VersionedDB> {
    vdb: V,
}



#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use statedb::*;
    use std::convert::{TryFrom, TryInto};
    use silk_proto::*;
    use error::*;
    use simulator::{BasedTxSimulator, TxSimulator};
    use rwset::validate::Validator;

    #[test]
    fn it_works() {
        let temp_dir = TempDir::new().unwrap();
        let provider = VersionedDBRocksProvider::new(temp_dir.into_path());
        let vdb = provider.get_db_handle("chain_id".to_string());
        let validate = Validator::new(vdb.clone());

        let mut sim = BasedTxSimulator::new("tx1".to_string(), vdb.clone());
        sim.set_state(&"contract_name".to_string(), &"k1".to_string(), Vec::from("v1")).unwrap();
        let results = sim.get_tx_simulation_results().unwrap();
        let tx = create_tx(results.simulation_results, 0).unwrap();
        let block = create_block(vec![tx], 1);

        let (batch, h) = validate.validate_and_prepare_batch(block).unwrap();
        println!("{:?}, {:?}", batch, h);
        vdb.apply_updates(batch, Some(h)).unwrap();


        let v1 = vdb.get_state(&"contract_name".to_string(), &"k1".to_string()).unwrap();
        assert_eq!(v1, Some(VersionedValue{
            value: Vec::from("v1"),
            metadata: vec![],
            version: Height { block_num: 1, tx_num: 0 }
        }))
    }

    fn create_block(txs: Vec<Transaction>, num: u64) -> Block {
        let data = txs.iter().map(|t|{
            utils::proto::marshal(t).unwrap()
        }).collect();

        Block{
            header: Some(BlockHeader{
                number: num,
                previous_hash: vec![],
                data_hash: vec![]
            }),
            data: Some(BlockData{ data, }),
            metadata: None
        }
    }
    fn create_tx(rw_set: TxReadWriteSet, tx_id: i32) -> Result<Transaction> {
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
                tx_id: format!("tx_id_{:?}", tx_id),
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

        let tx = Transaction{ signed_proposal: Some(sp), response: vec![proposal_response] };
        Ok(tx)
    }

    // #[test]
    // fn test_read_write_customer() {
    //     let temp_dir = TempDir::new().unwrap();
    //     let provider = VersionedDBRocksProvider::new(temp_dir.into_path());
    //     let vdb = provider.get_db_handle("chain_id".to_string());
    //
    //     {
    //         let mut sim = BasedTxSimulator::new("tx0".to_string(), vdb.clone());
    //         sim.set_state(&"ns".to_string(), &"key1".to_string(), Vec::from("value1")).unwrap();
    //         sim.set_state(&"ns".to_string(), &"key2".to_string(), Vec::from("value2")).unwrap();
    //         sim.set_state(&"ns".to_string(), &"key3".to_string(), Vec::from("value3")).unwrap();
    //         let results = sim.get_tx_simulation_results().unwrap();
    //
    //         let tx_rw_set= TxRwSet::try_from(results.simulation_results).unwrap();
    //         let code = validate_writeset(&tx_rw_set, vdb.clone()).unwrap();
    //         assert_eq!(code, TxValidationCode::Valid);
    //
    //         let h = Height{ block_num: 0, tx_num: 3 };
    //         let batch = apply_write_set(tx_rw_set, h.clone()).unwrap();
    //
    //         vdb.apply_updates(batch, Some(h)).unwrap();
    //     }
    //
    //     {
    //         let mut sim = BasedTxSimulator::new("tx1".to_string(), vdb.clone());
    //         let val_key1 = sim.get_state(&"ns".to_string(), &"key1".to_string()).unwrap();
    //         assert_eq!(val_key1, Vec::from("value1"));
    //         sim.set_state(&"ns".to_string(), &"key1".to_string(), Vec::from("1")).unwrap();
    //         let results = sim.get_tx_simulation_results().unwrap();
    //
    //         let tx_rw_set= TxRwSet::try_from(results.simulation_results).unwrap();
    //         let code = validate_writeset(&tx_rw_set, vdb.clone()).unwrap();
    //         assert_eq!(code, TxValidationCode::Valid);
    //
    //         let h = Height{ block_num: 0, tx_num: 3 };
    //         let batch = apply_write_set(tx_rw_set, h.clone()).unwrap();
    //         vdb.apply_updates(batch, Some(h)).unwrap();
    //
    //
    //
    //
    //
    //     }
    // }
}

