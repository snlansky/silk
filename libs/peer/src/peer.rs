use error::*;

use ledger::ledger_mgmt::LedgerMgr;

use ledger::{ledger_mgmt, LedgerProvider};
use silk_proto::Block;

pub struct Peer<P: LedgerProvider> {
    ledger_mgr: ledger_mgmt::LedgerMgr<P>,
}

impl<P: LedgerProvider> Peer<P> {
    pub fn new(ledger_mgr: LedgerMgr<P>) -> Self {
        Peer { ledger_mgr }
    }

    pub fn create_channel(_cid: &str, _block: Block) -> Result<()> {
        Ok(())
    }
}
