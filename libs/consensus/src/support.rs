use error::*;
use silk_proto::*;

use crate::{ChainSupport, IChain, IConsensus};
use tokio::sync::mpsc;

use dashmap::DashMap;
use silk_proto::message::MessageType;

pub struct Support {
    sender: mpsc::Sender<Message>,
    consensus: Box<dyn IConsensus>,
    chains: DashMap<String, Box<dyn IChain>>,
    chains_configure: DashMap<String, mpsc::Sender<BlockHeader>>,
}

impl Support {
    pub fn new(sender: mpsc::Sender<Message>, consensus: Box<dyn IConsensus>) -> Self {
        Support {
            sender,
            consensus,
            chains: DashMap::new(),
            chains_configure: DashMap::new(),
        }
    }

    pub async fn process_describe(&self, desc: ConsensusChainDescribe) -> Result<()> {
        let name = desc.chain.clone();
        let sender = self.chains_configure.get(&name);
        let header = desc
            .header
            .ok_or(from_str("ConsensusChainDescribe.Header is null"))?;

        match sender {
            Some(sender) => {
                let sender = &*sender;
                let mut sender = sender.clone();
                sender.send(header).await?;
            }
            None => {
                info!("create chain: {:}", &name);
                let (tx, mut rx) = mpsc::channel(10);
                self.chains_configure.insert(name.clone(), tx);
                let support = ChainSupport::new(name.clone(), header);

                let chain = self.consensus.handler_chain(support.clone());
                self.chains.insert(name, chain);

                tokio::spawn(async move {
                    while let Some(header) = rx.recv().await {
                        support.set(header).await;
                    }
                });
            }
        };
        Ok(())
    }

    pub fn process_message(&self, message: Message) -> Result<()> {
        let tx = utils::proto::unmarshal::<Transaction>(&message.content)?;
        let signed_prop = tx
            .signed_proposal
            .clone()
            .ok_or(from_str("signed proposal is null"))?;

        let header = utils::proto::unmarshal::<Proposal>(&signed_prop.proposal_bytes)?
            .header
            .unwrap();

        let chain = self
            .chains
            .get(&header.channel_id)
            .ok_or(from_str("process transaction. but chain not create"))?;
        let chain = &*chain;

        match header.header_type {
            t if t == HeaderType::Invoke as i32 => {
                chain.order(tx);
            }
            t if t == HeaderType::CreateChannel as i32 => {
                chain.configure(tx);
            }
            _ => {
                error!("unhandled header type {:?}", header.header_type);
            }
        }
        Ok(())
    }

    pub async fn handler(&self, msg: Message) -> Result<()> {
        info!("MESSAGE = {:?}", msg);
        match msg.message_type {
            t if t == MessageType::ConsensusChainDescribe as i32 => {
                let desc = utils::proto::unmarshal::<ConsensusChainDescribe>(&msg.content)?;
                info!("start update chain: {:?}", desc);
                self.process_describe(desc).await?;
            }
            t if t == MessageType::ConsensusTransactionArrived as i32 => {
                info!("process transaction");
                self.process_message(msg)?;
            }
            t if t == MessageType::Unregister as i32 => {
                // todo: close client
                return Err(from_str("get unregister message, consensus will close."));
            }
            _ => {
                error!("unhandled massage type {:?}", msg.message_type);
            }
        };
        Ok(())
    }
}
