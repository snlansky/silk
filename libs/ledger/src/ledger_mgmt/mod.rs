use crate::kvledger::kv_ledger_provider::Provider;
use crate::{Initializer, LedgerProvider};
use dashmap::DashMap;

use crate::statedb::VersionedDBRocksProvider;
use error::*;
use silk_proto::Block;
use std::sync::Arc;
use blockdb::provider::LevelDBBlockStoreProvider;

pub struct LedgerMgr<P: LedgerProvider> {
    opened_ledgers: DashMap<String, Arc<P::L>>,
    ledger_provider: P,
}

impl<P: LedgerProvider> LedgerMgr<P> {
    pub fn create_ledger(&self, id: &str, genesis_block: &Block) -> Result<Arc<P::L>> {
        let l = Arc::new(self.ledger_provider.create(genesis_block)?);
        debug!("create ledger {:?}", id);
        self.opened_ledgers.insert(String::from(id), l.clone());
        Ok(l)
    }

    pub fn open_ledger(&self, id: &str) -> Result<Arc<P::L>> {
        debug!("open ledger {:?}", id);
        if self.opened_ledgers.contains_key(id) {
            return Err(from_str(&format!("ledger {:?} already opened", id)));
        }

        let l = Arc::new(self.ledger_provider.open(id)?);
        self.opened_ledgers.insert(String::from(id), l.clone());
        Ok(l)
    }
}

impl LedgerMgr<Provider<VersionedDBRocksProvider, LevelDBBlockStoreProvider>> {
    pub fn new() -> Result<Self> {
        let init = Initializer {
            root_fs_path: "/var/silk/production".to_string(),
        };
        let vp = VersionedDBRocksProvider::new(&init.root_fs_path);
        let bsp = LevelDBBlockStoreProvider::new();
        let provider = Provider::new(init, vp, bsp)?;
        let l = LedgerMgr {
            opened_ledgers: DashMap::new(),
            ledger_provider: provider,
        };
        Ok(l)
    }
}
