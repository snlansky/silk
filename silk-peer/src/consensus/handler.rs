use silk_proto::*;

use super::*;

use silk_proto::message::MessageType;

// https://github.com/libp2p/rust-libp2p/blob/master/core/src/identity.rs

#[derive(Clone)]
pub struct Consensus {
    alg: String,
    sender: Sender<Message>, //mpsc::Sender<Result<Message, Status>>,
}

impl Consensus {
    pub fn new(register: ConsensusRegister, sender: Sender<Message>) -> Self {
        Consensus {
            alg: register.alg,
            sender,
        }
    }

    pub async fn send(&self, msg: Message) -> Result<()> {
        let sender = self.sender.clone();
        sender.send(Ok(msg)).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl IConsensus for Consensus {
    async fn handler(&self, msg: Message) -> Result<()> {
        debug!("received consensus message: {:?}", msg);
        match msg.message_type {
            t if t == MessageType::ConsensusNotifyBlockCommit as i32 => {
                let _block = utils::proto::unmarshal::<Block>(&msg.content)?;
                Ok(())
            }
            _ => {
                let describe = format!("unhandled massage type {:?}", msg.message_type);
                Err(from_str(&describe))
            }
        }
    }

    async fn broadcast(&self, tx: &Transaction) -> Result<()> {
        let msg = Message {
            message_type: MessageType::ConsensusTransactionArrived as i32,
            correlation_id: "".to_string(),
            content: utils::proto::marshal(tx).map_err(Box::new)?,
        };
        self.send(msg).await
    }

    async fn notify_update_chain(&self, chain: String, block: &Block) -> Result<()> {
        let desc = ConsensusChainDescribe {
            chain: chain.clone(),
            header: block.header.clone(),
        };

        let msg = Message {
            message_type: MessageType::ConsensusChainDescribe as i32,
            correlation_id: chain,
            content: utils::proto::marshal(&desc).map_err(Box::new)?,
        };
        self.send(msg).await
    }

    fn close(&mut self) {
        unimplemented!()
    }
}
