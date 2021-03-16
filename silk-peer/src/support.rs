use crate::contract::*;
use error::*;

use crate::consensus::*;
use silk_proto::*;

#[async_trait::async_trait]
pub trait ISupport: Send + Sync + 'static {
    fn get_transaction_simulator(&self, ledger: &str, tx_id: &str) -> Result<Option<TxSimulator>>;
    fn get_ledger_height(&self, channel_id: String) -> Result<u64>;
    fn get_transaction_by_id(&self, channel_id: String, _tx_id: String) -> Result<()>;
    async fn execute<'a>(
        &self,
        tx_params: &'a TransactionParams,
        name: &'a str,
    ) -> Result<(Response, Option<ContractEvent>)>;
    async fn broadcast(&self, tx: &Transaction) -> Result<()>;
}

#[derive(Clone)]
pub struct Support<H: IContractSupport, T: IConsensusSupport> {
    contract_support: H,
    consensus_support: T,
    node_support: i32,
}

impl<H: IContractSupport, T: IConsensusSupport> Support<H, T> {
    pub fn new(contract_support: H, consensus_support: T) -> Self {
        Support {
            contract_support,
            consensus_support,
            node_support: 0,
        }
    }
}

#[async_trait::async_trait]
impl<H: IContractSupport, T: IConsensusSupport> ISupport for Support<H, T> {
    // PIN
    fn get_transaction_simulator(
        &self,
        _ledger: &str,
        _tx_id: &str,
    ) -> Result<Option<TxSimulator>> {
        Ok(Some(TxSimulator {}))
    }

    // PIN
    fn get_ledger_height(&self, _channel_id: String) -> Result<u64> {
        Ok(0)
    }

    // PIN
    fn get_transaction_by_id(&self, _channel_id: String, _tx_id: String) -> Result<()> {
        Ok(())
    }

    // PIN
    async fn execute<'a>(
        &self,
        tx_params: &'a TransactionParams,
        name: &'a str,
    ) -> Result<(Response, Option<ContractEvent>)> {
        self.contract_support.execute(tx_params, name).await
    }

    async fn broadcast(&self, tx: &Transaction) -> Result<()> {
        self.consensus_support.commit(tx).await
    }
}
