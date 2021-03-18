use ledger::kvledger::kv_ledger_provider::Provider;
use ledger::ledger_mgmt;
use ledger::statedb::VersionedDBRocksProvider;

pub struct Peer {
    ledger_mgr: ledger_mgmt::LedgerMgr<Provider<VersionedDBRocksProvider>>,
}

impl Peer {
    pub fn new() -> Self {
        Self {
            ledger_mgr: ledger_mgmt::new().unwrap(),
        }
    }
}
