use crate::{Ledger, LedgerProvider};
use dashmap::DashMap;
use crate::kvledger::kv_ledger_provider::Provider;
use crate::kvledger::kv_ledger::KVLedger;
use error::*;
use silk_proto::Block;

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
        Ok(l)
    }
}

pub fn new() -> LedgerMgr<Provider> {
    let provider = Provider::new();
    LedgerMgr::new(provider)
}
