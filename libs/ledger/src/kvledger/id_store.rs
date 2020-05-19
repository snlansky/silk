use rocksdb::DB;
use error::*;

pub struct IDStore {
    db: DB
}

impl IDStore {
    pub fn new(path: &String) -> Result<Self> {
        Ok(IDStore{db: rocksdb::DB::open_default(path)?})
    }
}
