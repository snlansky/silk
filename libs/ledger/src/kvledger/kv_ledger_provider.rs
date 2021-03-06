use crate::kvledger::id_store::IDStore;
use crate::kvledger::kv_ledger::KVLedger;
use crate::statedb::VersionedDBProvider;
use crate::Initializer;
use blockdb::BlockStoreProvider;
use error::*;
use silk_proto::Block;
use utils::utils;

pub struct Provider<VP: VersionedDBProvider, BSP: BlockStoreProvider> {
    id_store: IDStore,
    block_store_provider: BSP,
    vdb_provider: VP,
    history_db_provider: String,
}

impl<VP: VersionedDBProvider, BSP: BlockStoreProvider> Provider<VP, BSP> {
    pub fn new(init: Initializer, vdb_provider: VP, block_store_provider: BSP) -> Result<Self> {
        let p = Provider {
            id_store: IDStore::new(&init.root_fs_path)?,
            block_store_provider,
            vdb_provider,
            history_db_provider: "".to_string(),
        };

        Ok(p)
    }
}

impl<VP: VersionedDBProvider, BSP: BlockStoreProvider> crate::LedgerProvider for Provider<VP, BSP> {
    type L = KVLedger;

    fn create(&self, genesis_block: &Block) -> Result<Self::L> {
        let ledger_id = utils::get_chain_id_from_block(genesis_block)?;
        if self.id_store.ledger_id_exists(&ledger_id)? {
            return Err(from_str(format!("ledger {:} exist", ledger_id).as_str()));
        }

        self.id_store.create_ledger_id(&ledger_id, genesis_block)?;

        // TODO: init block store
        // TODO: init history db
        let _vdb = self.vdb_provider.get_db_handle(&ledger_id);

        let kvl = KVLedger::new();

        Ok(kvl)
    }

    fn open(&self, _ledger_id: &str) -> Result<Self::L> {
        unimplemented!()
    }

    fn exists(&self, _ledger_id: &str) -> Result<bool> {
        unimplemented!()
    }

    fn list(&self) -> Result<Vec<String>> {
        unimplemented!()
    }

    fn close(&self) {
        unimplemented!()
    }
}
