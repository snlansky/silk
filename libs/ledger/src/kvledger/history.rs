use crate::statedb::ResultsIterator;
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
    type Iter = HistoryQueryResultsIterator;

    fn get_history_for_key(_namespace: String, _key: String) -> Result<Self::Iter> {
        unimplemented!()
    }
}

pub struct HistoryQueryResultsIterator {}

impl HistoryQueryResultsIterator {
    pub fn new() -> Self {
        HistoryQueryResultsIterator {}
    }
}

impl ResultsIterator<KeyModification> for HistoryQueryResultsIterator {
    fn next(&self) -> Result<KeyModification> {
        unimplemented!()
    }

    fn close(&self) {
        unimplemented!()
    }
}
