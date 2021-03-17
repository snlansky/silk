use error::*;
use tonic::transport::Channel;

use silk_proto::consensus_client::ConsensusClient;

use crate::support::Support;
use crate::IConsensus;
use silk_proto::ConsensusRegister;

use tokio::sync::mpsc;
use tonic::Request;

pub async fn start<T: IConsensus>(
    client: &mut ConsensusClient<Channel>,
    name: &str,
    consensus: T,
) -> Result<()> {
    let reg = ConsensusRegister {
        alg: name.to_string(),
        decorations: Default::default(),
    };
    let msg = silk_proto::Message {
        message_type: silk_proto::message::MessageType::ConsensusRegister as i32,
        correlation_id: "".to_string(),
        content: utils::proto::marshal(&reg).unwrap(),
    };

    let (tx, rx) = mpsc::channel(1000);
    // send register message to peer
    tx.send(msg).await?;

    let response = client.register(Request::new(rx)).await?;
    let mut inbound = response.into_inner();

    let support = Support::new(tx, consensus);
    while let Some(msg) = inbound.message().await.unwrap() {
        support.handler(msg).await?;
    }

    Ok(())
}
