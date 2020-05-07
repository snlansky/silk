use consensus::*;
use error::*;
use silk_proto::consensus_client::ConsensusClient;
use silk_proto::*;

#[macro_use]
extern crate log;

struct SoloConsensus {
    alg: String,
}

impl SoloConsensus {
    fn new() -> Self {
        SoloConsensus {
            alg: "solo".to_string(),
        }
    }
}

impl IConsensus for SoloConsensus {
    type Output = Chain;

    fn handler_chain<S: IChainSupport>(&self, support: S) -> Self::Output {
        Chain{ support: Box::new(support) }
    }
}

struct Chain {
    support: Box<dyn IChainSupport>,
}

impl IChain for Chain {
    fn configure(&self, _tx: Transaction) -> Result<()> {
        unimplemented!()
    }

    fn order(&self, _tx: Transaction) -> Result<()> {
        info!("commit new transaction");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let mut client = ConsensusClient::connect("http://127.0.0.1:8081").await?;
    start(&mut client, "solo", SoloConsensus::new()).await?;
    Ok(())
}
