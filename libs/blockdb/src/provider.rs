use crate::store::Store;

use error::*;

pub struct LevelDBBlockStoreProvider {}

impl LevelDBBlockStoreProvider {
    pub fn new() -> Self {
        unimplemented!()
    }
}

impl crate::BlockStoreProvider for LevelDBBlockStoreProvider {
    type S = Store;

    fn create_block_store(_ledger_id: &str) -> Result<Self::S> {
        unimplemented!()
    }

    fn open_block_store(_ledger_id: &str) -> Result<Self::S> {
        unimplemented!()
    }

    fn exists(_ledger_id: &str) -> Result<bool> {
        unimplemented!()
    }

    fn list() -> Result<Vec<String>> {
        unimplemented!()
    }

    fn close() {
        unimplemented!()
    }
}
