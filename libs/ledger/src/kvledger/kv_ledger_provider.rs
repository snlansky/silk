use silk_proto::Block;
use error::*;
use crate::kvledger::kv_ledger::KVLedger;
use utils::utils::get_chain_id_from_block;
use crate::statedb::{VersionedDBProvider, VersionedDBRocksProvider, VersionedDB, ResultsIterator, VersionedKV};
use crate::Initializer;
use crate::kvledger::id_store::IDStore;

pub struct Provider<VP: VersionedDBProvider>{
    id_store: IDStore,
    block_store_provider: String,
    vdb_provider: VP,
    history_db_provider: String,

}

impl <VP: VersionedDBProvider>Provider<VP> {
    pub fn new(init :Initializer, provider: VP) -> Result<Self> {

        let p = Provider{
            id_store: IDStore::new(&init.root_fs_path)?,
            block_store_provider: "".to_string(),
            vdb_provider: provider,
            history_db_provider: "".to_string()
        };

        Ok(p)
    }
}

impl <VP: VersionedDBProvider>crate::LedgerProvider for Provider<VP> {
    type L = KVLedger;

    fn create(&self, genesis_block: &Block) -> Result<Self::L> {
        let ledger_id = get_chain_id_from_block(genesis_block)?;
        unimplemented!()
    }

    fn open(&self, ledger_id: String) -> Result<Self::L> {
        unimplemented!()
    }

    fn exists(&self, ledger_id: String) -> Result<bool> {
        unimplemented!()
    }

    fn list(&self) -> Result<Vec<String>> {
        unimplemented!()
    }

    fn close(&self) {
        unimplemented!()
    }
}
