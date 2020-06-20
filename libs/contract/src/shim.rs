use std::error::Error;

use tonic::transport::Channel;

use silk_proto::contract_client::ContractClient;
use silk_proto::*;
use tokio::sync::mpsc;

use super::Contract;
use crate::ContractStub;
use silk_proto::message::MessageType;
use std::sync::Arc;
use tonic::Request;

async fn run_register(
    client: &mut ContractClient<Channel>,
    name: String,
    contract: Arc<Box<dyn Contract>>,
) -> Result<(), Box<dyn Error>> {
    let (mut tx, rx) = mpsc::channel(1000);

    let reg = ContractRegister {
        name,
        decorations: Default::default(),
    };

    let msg = silk_proto::Message {
        message_type: silk_proto::message::MessageType::ContractRegister as i32,
        correlation_id: "".to_string(),
        content: utils::proto::marshal(&reg).unwrap(),
    };
    // send register message to peer
    tx.send(msg).await?;

    let response = client.register(Request::new(rx)).await?;
    let mut inbound = response.into_inner();

    while let Some(msg) = inbound.message().await.unwrap() {
        println!("MESSAGE = {:?}", msg);
        match msg.message_type {
            t if t == message::MessageType::ContractTransaction as i32 => {
                println!("start transaction ...");
                let ct: ContractTransaction =
                    utils::proto::unmarshal(msg.content.as_slice()).unwrap();
                let ret = start_transaction(client, contract.clone(), ct);
                let m = Message {
                    message_type: MessageType::ContractTransactionCompletedRequest as i32,
                    correlation_id: msg.correlation_id,
                    content: utils::proto::marshal(&ret).map_err(Box::new)?,
                };
                tx.send(m).await?
            }
            t if t == message::MessageType::Unregister as i32 => {
                // todo: close client
                return Err(Box::<dyn Error>::from(
                    "get unregister message, contract will close.".to_string(),
                ));
            }
            _ => {
                println!("unhandled massage type {:?}", msg.message_type);
            }
        }
    }

    Ok(())
}

fn start_transaction(
    client: &mut ContractClient<Channel>,
    contract: Arc<Box<dyn Contract>>,
    tx: ContractTransaction,
) -> TransactionCompleted {
    let proposal = tx.proposal.unwrap();
    let mut stub = ContractStub::new(client, proposal.clone());
    let resp = contract.invoke(&mut stub);
    TransactionCompleted {
        proposal: Some(proposal),
        response: Some(resp),
        event: stub.get_event(),
    }
}

pub async fn start(
    name: String,
    contract: Box<dyn Contract>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ContractClient::connect("http://127.0.0.1:8081").await?;

    run_register(&mut client, name, Arc::new(contract)).await?;

    Ok(())
}
