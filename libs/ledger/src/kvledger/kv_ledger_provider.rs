use silk_proto::Block;
use error::*;
use crate::kvledger::kv_ledger::KVLedger;

pub struct Provider {

}

impl Provider {
    pub fn new() -> Self {
        Provider{}
    }
}

impl crate::LedgerProvider for Provider {
    type L = KVLedger;

    fn create(&self, genesis_block: &Block) -> Result<Self::L> {
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
