use crate::HistoryQueryExecutor;
use error::*;
use silk_proto::KeyModification;

pub struct KVHistoryQueryExecutor {}

impl KVHistoryQueryExecutor {
    pub fn new() -> Self {
        KVHistoryQueryExecutor {}
    }
}

impl HistoryQueryExecutor for KVHistoryQueryExecutor {
    fn get_history_for_key(_namespace: String, _key: String) -> Result<Box<dyn Iterator<Item=KeyModification>>> {
        unimplemented!()
    }
}

