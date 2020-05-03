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
    use statedb::{VersionedDBRocksProvider, VersionedDBProvider, Height, VersionedDB, VersionedValue};
    use simulator::{BasedTxSimulator, TxSimulator};
    use rwset::validate::validate_writeset;
    use std::convert::{TryFrom, TryInto};
    use rwset::builder::*;
    use silk_proto::TxValidationCode;
    use rwset::key::apply_write_set;

    #[test]
    fn it_works() {
        let temp_dir = TempDir::new().unwrap();
        let provider = VersionedDBRocksProvider::new(temp_dir.into_path());
        let vdb = provider.get_db_handle("chain_id".to_string());

        let mut sim = BasedTxSimulator::new("tx1".to_string(), vdb.clone());

        sim.set_state(&"contract_name".to_string(), &"k1".to_string(), Vec::from("v1")).unwrap();

        let results = sim.get_tx_simulation_results().unwrap();

        let tx_rw_set= TxRwSet::try_from(results.simulation_results).unwrap();
        let code = validate_writeset(&tx_rw_set, vdb.clone()).unwrap();
        assert_eq!(code, TxValidationCode::Valid);

        let h = Height{ block_num: 0, tx_num: 1 };
        let batch = apply_write_set(tx_rw_set, h.clone()).unwrap();

        vdb.apply_updates(batch, Some(h)).unwrap();

        let v1 = vdb.get_state(&"contract_name".to_string(), &"k1".to_string()).unwrap();
        assert_eq!(v1, Some(VersionedValue{
            value: Vec::from("v1"),
            metadata: vec![],
            version: Height { block_num: 0, tx_num: 1 }
        }))
    }

    #[test]
    fn test_read_write_customer() {
        let temp_dir = TempDir::new().unwrap();
        let provider = VersionedDBRocksProvider::new(temp_dir.into_path());
        let vdb = provider.get_db_handle("chain_id".to_string());

        {
            let mut sim = BasedTxSimulator::new("tx0".to_string(), vdb.clone());
            sim.set_state(&"ns".to_string(), &"key1".to_string(), Vec::from("value1")).unwrap();
            sim.set_state(&"ns".to_string(), &"key2".to_string(), Vec::from("value2")).unwrap();
            sim.set_state(&"ns".to_string(), &"key3".to_string(), Vec::from("value3")).unwrap();
            let results = sim.get_tx_simulation_results().unwrap();

            let tx_rw_set= TxRwSet::try_from(results.simulation_results).unwrap();
            let code = validate_writeset(&tx_rw_set, vdb.clone()).unwrap();
            assert_eq!(code, TxValidationCode::Valid);

            let h = Height{ block_num: 0, tx_num: 3 };
            let batch = apply_write_set(tx_rw_set, h.clone()).unwrap();

            vdb.apply_updates(batch, Some(h)).unwrap();
        }

        {
            let mut sim = BasedTxSimulator::new("tx1".to_string(), vdb.clone());
            let val_key1 = sim.get_state(&"ns".to_string(), &"key1".to_string()).unwrap();
            assert_eq!(val_key1, Vec::from("value1"));
            sim.set_state(&"ns".to_string(), &"key1".to_string(), Vec::from("1")).unwrap();
            let results = sim.get_tx_simulation_results().unwrap();

            let tx_rw_set= TxRwSet::try_from(results.simulation_results).unwrap();
            let code = validate_writeset(&tx_rw_set, vdb.clone()).unwrap();
            assert_eq!(code, TxValidationCode::Valid);

            let h = Height{ block_num: 0, tx_num: 3 };
            let batch = apply_write_set(tx_rw_set, h.clone()).unwrap();
            vdb.apply_updates(batch, Some(h)).unwrap();





        }
    }
}
