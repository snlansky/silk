use error::*;
use ledger::kvledger::kv_ledger_provider::Provider;
use ledger::ledger_mgmt::LedgerMgr;
use ledger::statedb::VersionedDBRocksProvider;
use ledger::{ledger_mgmt, LedgerProvider};
use silk_proto::Block;

pub struct Peer<P: LedgerProvider> {
    ledger_mgr: ledger_mgmt::LedgerMgr<P>,
}

impl<P: LedgerProvider> Peer<P> {
    pub fn new(ledger_mgr: LedgerMgr<P>) -> Self {
        Peer { ledger_mgr }
    }

    pub fn create_channel(cid: &str, block: Block) -> Result<()> {
        Ok(())
    }
}
