use silk_proto::*;

use super::*;
use std::sync::Arc;

use dashmap::DashMap;
use silk_proto::message::MessageType;
use std::time::Duration;

// https://github.com/libp2p/rust-libp2p/blob/master/core/src/identity.rs

#[derive(Clone)]
pub struct Contract {
    name: String,
    sender: Sender<Message>,
    transaction_context_registry: Arc<DashMap<String, TransactionContext>>, // key: channel_id + tx_id
}

impl Contract {
    pub fn new(register: ContractRegister, sender: Sender<Message>) -> Self {
        Contract {
            name: register.name,
            sender,
            transaction_context_registry: Arc::new(DashMap::new()),
        }
    }

    pub fn contract_id(&self) -> String {
        self.name.clone()
    }

    fn notify(&self, msg: &Message) -> Result<()> {
        let TransactionCompleted {
            proposal,
            response,
            event,
        } = utils::proto::unmarshal(&msg.content)?;
        let prop = proposal.ok_or(from_str("proposal is null"))?;

        if let Some(ctx) = self
            .transaction_context_registry
            .remove(&msg.correlation_id)
        {
            let ctx = ctx.1;

            ctx.response_notifier
                .send(TransactionCompleted {
                    proposal: Some(prop),
                    response,
                    event,
                })
                .map_err(|_| from_str("send error"))?;
        }
        Ok(())
    }

    fn handle_get_state(
        &self,
        _ctx: &TransactionContext,
        _msg: &Message,
    ) -> Result<Option<Message>> {
        Ok(None)
    }

    fn handle_put_state(
        &self,
        _ctx: &TransactionContext,
        _msg: &Message,
    ) -> Result<Option<Message>> {
        Ok(None)
    }

    async fn handle_transaction<F>(&self, msg: &Message, delegate: F) -> Result<()>
    where
        F: Fn(&TransactionContext, &Message) -> Result<Option<Message>>,
    {
        let ctx = self
            .transaction_context_registry
            .get(&msg.correlation_id)
            .ok_or(from_str("transaction not found"))?;
        let ctx = &*ctx;
        if let Some(m) = delegate(ctx, msg)? {
            let mut sender = self.sender.clone();
            sender.send(Ok(m)).await?;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Handler for Contract {
    async fn handler(&self, msg: Message) -> Result<()> {
        debug!("received contract message: {:?}", msg);
        match msg.message_type {
            t if t == MessageType::ContractTransactionCompletedRequest as i32 => self.notify(&msg),
            t if t == MessageType::ContractGetStateRequest as i32 => {
                let delegate =
                    |ctx: &TransactionContext, msg: &Message| self.handle_get_state(ctx, msg);
                self.handle_transaction(&msg, delegate).await
            }
            _ => {
                let describe = format!("unhandled massage type {:?}", msg.message_type);
                Err(from_str(&describe))
            }
        }
    }

    async fn execute(
        &self,
        tx_params: &TransactionParams,
        msg: Message,
        _timeout: Duration,
    ) -> Result<TransactionCompleted> {
        let mut sender = self.sender.clone();
        let _registry = self.transaction_context_registry.clone();

        let key = msg.correlation_id.clone();
        sender.send(Ok(msg)).await?;

        let (tx, rx) = tokio::sync::oneshot::channel();

        let tx_ctx = TransactionContext {
            channel_id: tx_params.channel_id.clone(),
            namespace: tx_params.namespace.clone(),
            proposal: tx_params.proposal.clone(),
            response_notifier: tx,
            simulator: tx_params.tx_simulator.clone(),
        };

        self.transaction_context_registry.insert(key, tx_ctx);
        debug!("wait transaction completed message from contract");
        Ok(rx.await?)
    }

    fn close(&mut self) {
        unimplemented!()
    }
}
