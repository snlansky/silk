mod server;
pub use server::*;

mod handler;
use error::*;
pub use handler::*;
use silk_proto::*;

mod support;
pub use support::*;

const SYSTEM_CHANNEL: &str = "system_channel";

#[async_trait::async_trait]
pub trait IConsensus: Send + Sync + 'static {
    async fn handler(&self, msg: Message) -> Result<()>;
    async fn broadcast(&self, tx: &Transaction) -> Result<()>;
    async fn notify_update_chain(&self, channel: String, block: &Block) -> Result<()>;
    fn close(&mut self);
}

#[async_trait::async_trait]
pub trait IConsensusSupport: Send + Sync + 'static {
    async fn register(&self, consensus: Consensus) -> Result<()>;
    async fn deregister(&self, name: &String) -> Result<()>;
    async fn commit(&self, tx: &Transaction) -> Result<()>;
    async fn update_chain(&self) -> Result<()>;
}
