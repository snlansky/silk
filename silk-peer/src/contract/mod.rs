mod server;
pub use server::*;

mod handler;
mod support;
use dashmap::mapref::one::Ref;
use error::*;
pub use handler::*;
use silk_proto::*;
pub use support::*;

// https://github.com/libp2p/rust-libp2p/blob/master/core/src/identity.rs
#[async_trait::async_trait]
pub trait Handler: Send + Sync + 'static {
    async fn handler(&self, msg: Message) -> Result<()>;
    async fn execute(
        &self,
        tx_params: &TransactionParams,
        msg: Message,
        timeout: std::time::Duration,
    ) -> Result<TransactionCompleted>;
    fn close(&mut self);
}

#[async_trait::async_trait]
pub trait IContractSupport: Send + Sync + 'static {
    // PIN
    fn register(&self, contract: Contract) -> Result<()>;
    // PIN
    fn deregister(&self, name: &String) -> Result<()>;
    // PIN
    fn launch(&self, name: &String) -> Option<Ref<String, Contract>>;
    // PIN
    async fn execute(
        &self,
        tx_params: &TransactionParams,
        contract: &String,
    ) -> Result<(Response, Option<ContractEvent>)>;
    // PIN
    async fn invoke(
        &self,
        tx_params: &TransactionParams,
        contract: &String,
    ) -> Result<TransactionCompleted>;
}

pub struct TransactionParams {
    pub tx_id: String,
    pub channel_id: String,
    pub namespace: String,
    //  pub signed_proposal: &'a SignedProposal,
    pub proposal: Proposal,
    pub tx_simulator: TxSimulator,
}

#[derive(Clone)]
pub struct TxSimulator {}

impl TxSimulator {
    pub fn get_tx_simulation_results(&self) -> Result<TxReadWriteSet> {
        Ok(TxReadWriteSet::default())
    }
}

pub struct TransactionContext {
    channel_id: String,
    namespace: String,
    proposal: Proposal,
    response_notifier: tokio::sync::oneshot::Sender<TransactionCompleted>,
    simulator: TxSimulator,
}
