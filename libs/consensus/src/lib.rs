use error::*;
use tonic::transport::Channel;

use silk_proto::consensus_client::ConsensusClient;
use silk_proto::*;

mod support;
pub use support::*;

mod start;
pub use start::*;

#[macro_use]
extern crate log;
use std::sync::Arc;
use tokio::sync::RwLock;

pub trait IChain {
    fn configure(&self, tx: Transaction) -> Result<()>;
    fn order(&self, tx: Transaction) -> Result<()>;
}

#[derive(Clone)]
pub struct ChainSupport {
    id: String,
    header: Arc<RwLock<BlockHeader>>,
}

impl ChainSupport {
    pub fn new(id: String, header: BlockHeader) -> Self {
        ChainSupport {
            id,
            header: Arc::new(RwLock::new(header)),
        }
    }

    pub async fn set(&self, header: BlockHeader) {
        let mut lock = self.header.write().await;

        if lock.number != header.number {
            info!("update chain {:} header {:}", self.id, header.number);
            *lock = header
        }
    }
}

impl ChainSupport {
    pub fn create_next_block(&self, _vec: Vec<Transaction>) -> Block {
        unimplemented!()
    }
    pub fn commit_block(&self, _block: Block) -> Result<()> {
        unimplemented!()
    }

    pub async fn start(&self) -> Result<()> {
        let lock = self.header.read().await;
        println!("->{:}", lock.number);
        Ok(())
    }
}

pub trait IConsensus: Send + Sync + 'static {
    fn handler_chain(&self, support: ChainSupport) -> Box<dyn IChain>;
}

pub struct ConsensusStub<'a> {
    client: &'a ConsensusClient<Channel>,
}

#[cfg(test)]
mod tests {
    use crate::ChainSupport;
    use futures::executor::block_on;
    use silk_proto::BlockHeader;

    #[test]
    fn it_works() {
        let s = ChainSupport::new(
            "default".to_string(),
            BlockHeader {
                number: 10,
                previous_hash: vec![],
                data_hash: vec![],
            },
        );

        let sc = s.clone();

        let set = sc.set(BlockHeader {
            number: 11,
            previous_hash: vec![],
            data_hash: vec![],
        });
        block_on(set);

        let f = s.start();
        block_on(f);

        let f2 = sc.start();
        block_on(f2);
    }
}
