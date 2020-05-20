use crate::{Ledger, LedgerProvider, Initializer};
use dashmap::DashMap;
use crate::kvledger::kv_ledger_provider::Provider;
use crate::kvledger::kv_ledger::KVLedger;
use error::*;
use silk_proto::Block;
use crate::statedb::VersionedDBRocksProvider;

pub struct LedgerMgr<P: LedgerProvider> {
    opened_ledgers: DashMap<String, P::L>,
    ledger_provider: P,
}

impl <P: LedgerProvider>LedgerMgr<P>{
    fn new(provider : P) -> Self {
        LedgerMgr{ opened_ledgers: DashMap::new(), ledger_provider: provider }
    }

    pub fn create_ledger(&self, id: String, genesis_block: &Block) -> Result<P::L> {
        let l= self.ledger_provider.create(genesis_block)?;
        // TODO: insert opened_ledgers
        // self.opened_ledgers.insert(id, l);
        Ok(l)
    }

    pub fn open_ledger(&self, id: String) -> Result<Option<P::L>> {
        unimplemented!()
    }
}

pub fn new() -> Result<LedgerMgr<Provider<VersionedDBRocksProvider>>> {
    let init = Initializer{root_fs_path:"var/silk/production".to_string()};
    let vp = VersionedDBRocksProvider::new(&init.root_fs_path);
    let provider = Provider::new(init, vp)?;
    Ok(LedgerMgr::new(provider))
}
