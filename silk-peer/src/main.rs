#[macro_use]
extern crate failure;
use futures::future;

#[macro_use]
extern crate log;

use silk_proto::consensus_server::ConsensusServer;
use silk_proto::contract_server::ContractServer;
use silk_proto::endorser_server::EndorserServer;

use tonic::transport::Server;

use crate::consensus::ConsensusSupport;
use crate::contract::ContractSupport;
use crate::event::EventHandler;

pub mod channel;
pub mod consensus;
pub mod contract;
pub mod endorser;
pub mod event;
pub mod p2p;
pub mod support;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let access_addr = "127.0.0.1:8080".parse().unwrap();
    let inner_addr = "127.0.0.1:8081".parse().unwrap();

    let handler = Box::new(EventHandler::new());

    let contract_support = ContractSupport::new();
    let consensus_support = ConsensusSupport::new(handler);
    let support = support::Support::new(contract_support.clone(), consensus_support.clone());

    let consensus_svr = consensus::Server::new(consensus_support);
    let contract_svr = contract::Server::new(contract_support);
    let endorser_svr = endorser::Server::new(support);

    info!("component server listening on {}", inner_addr);
    info!("sdk server listening on {}", access_addr);

    let c_svr = Server::builder()
        .add_service(ConsensusServer::new(consensus_svr))
        .add_service(ContractServer::new(contract_svr))
        .serve(inner_addr);

    let s_svr = Server::builder()
        .add_service(EndorserServer::new(endorser_svr))
        .serve(access_addr);

    future::try_join(c_svr, s_svr).await?;
    Ok(())
}
