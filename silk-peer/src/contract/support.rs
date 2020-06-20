use super::*;
use std::sync::Arc;

use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use failure::_core::time::Duration;
use silk_proto::message::MessageType;

#[derive(Clone, Default)]
pub struct ContractSupport {
    handler_registry: Arc<DashMap<String, Contract>>,
}

#[async_trait::async_trait]
impl IContractSupport for ContractSupport {
    // PIN
    fn register(&self, contract: Contract) -> Result<()> {
        self.handler_registry
            .insert(contract.contract_id(), contract);
        Ok(())
    }

    // PIN
    fn deregister(&self, _name: &str) -> Result<()> {
        unimplemented!()
    }

    // PIN
    fn launch(&self, name: &str) -> Option<Ref<String, Contract>> {
        self.handler_registry.get(name)
    }

    // PIN
    async fn execute(
        &self,
        tx_params: &TransactionParams,
        contract: &str,
    ) -> Result<(Response, Option<ContractEvent>)> {
        let msg = self.invoke(tx_params, contract).await?;
        let resp = msg.response.ok_or_else(||from_str("contract response is null"))?;
        Ok((resp, msg.event))
    }

    // PIN: contract invoke interface
    async fn invoke(
        &self,
        tx_params: &TransactionParams,
        contract: &str,
    ) -> Result<TransactionCompleted> {
        let ct = ContractTransaction {
            proposal: Some(tx_params.proposal.clone()),
        };
        let payload = utils::proto::marshal(&ct).map_err(Box::new)?;
        let msg = Message {
            message_type: MessageType::ContractTransaction as i32,
            correlation_id: tx_params.channel_id.clone() + &tx_params.tx_id,
            content: payload,
        };
        let r = self
            .launch(contract)
            .ok_or_else(||from_str("contract not found"))?;
        let h = &*r;
        h.execute(tx_params, msg, Duration::from_secs(5)).await
    }
}
