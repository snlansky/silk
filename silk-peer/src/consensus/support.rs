use super::*;
use std::sync::Arc;

use crate::event::Handler;

use tokio::sync::RwLock;

#[derive(Clone)]
pub struct ConsensusSupport {
    handler_registry: Arc<RwLock<Option<Consensus>>>,
    eh: Arc<Box<dyn Handler>>,
}

impl ConsensusSupport {
    pub fn new(eh: Box<dyn Handler>) -> ConsensusSupport {
        ConsensusSupport {
            handler_registry: Arc::new(RwLock::new(None)),
            eh: Arc::new(eh),
        }
    }
}

#[async_trait::async_trait]
impl IConsensusSupport for ConsensusSupport {
    async fn register(&self, consensus: Consensus) -> Result<()> {
        let mut lock = self.handler_registry.write().await;
        *lock = Some(consensus);
        Ok(())
    }

    async fn deregister(&self, _name: &str) -> Result<()> {
        unimplemented!()
    }

    async fn commit(&self, tx: &Transaction) -> Result<()> {
        info!("commit -> {:?}", tx);
        let lock = self.handler_registry.read().await;

        match lock.as_ref() {
            Some(c) => {
                c.broadcast(tx).await?;
                Ok(())
            }
            None => Err(from_str("consensus not setup")),
        }
    }

    async fn update_chain(&self) -> Result<()> {
        let lock = self.handler_registry.read().await;
        let lock = lock.as_ref().unwrap();
        lock.notify_update_chain(
            SYSTEM_CHANNEL.to_string(),
            &Block {
                header: Some(BlockHeader {
                    number: 0,
                    previous_hash: vec![],
                    data_hash: vec![],
                }),
                data: None,
                metadata: None,
            },
        )
        .await
    }
}
