#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use statedb::{VersionedDBRocksProvider, VersionedDBProvider};
    use simulator::BasedTxSimulator;

    #[test]
    fn it_works() {
        let temp_dir = TempDir::new().unwrap();
        let provider = VersionedDBRocksProvider::new(temp_dir.into_path());
        let vdb = provider.get_db_handle("chain_id".to_string());

        let sim = BasedTxSimulator::new("tx1".to_string(), *vdb);
    }
}
