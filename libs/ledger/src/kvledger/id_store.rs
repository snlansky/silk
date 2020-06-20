use byteorder::WriteBytesExt;
use error::*;
use rocksdb::DB;
use silk_proto::Block;
use std::io::Write;
use std::path::PathBuf;

pub struct IDStore {
    db: DB,
}

impl IDStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let path = path.join("ledger_provider");
        Ok(IDStore {
            db: rocksdb::DB::open_default(path)?,
        })
    }

    pub fn create_ledger_id(&self, ledger_id: &str, block: &Block) -> Result<()> {
        let key = self.encode_ledger_key(ledger_id);
        if self.db.get(&key)?.is_some() {
            return Err(from_str(format!("ledger {:} exist", ledger_id).as_str()));
        }

        self.db.put(key, utils::proto::marshal(block)?)?;
        Ok(())
    }

    pub fn ledger_id_exists(&self, ledger_id: &str) -> Result<bool> {
        let key = self.encode_ledger_key(&ledger_id);
        let v = self.db.get(key)?;
        Ok(v.is_some())
    }

    fn encode_ledger_key(&self, ledger_id: &str) -> Vec<u8> {
        let mut buf = vec![];
        buf.write_u8(b'l').unwrap();
        let _ = buf.write(ledger_id.as_bytes()).unwrap();
        buf
    }
}
