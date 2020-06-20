use crate::contract::*;
use crate::support::ISupport;
use silk_proto::endorser_server::Endorser;
use silk_proto::*;
use tonic::Request;

use error::*;

#[derive(Clone)]
pub struct Server<S: ISupport> {
    support: S,
}

impl<S: ISupport> Server<S> {
    pub fn new(support: S) -> Self {
        Server { support }
    }

    // 验证签名，参数等
    fn verify(&self, _sp: &SignedProposal, _creator: &[u8]) -> Result<()> {
        Ok(())
    }

    async fn process(&self, signed_proposal: SignedProposal) -> Result<ProposalResponse> {
        let proposal: Proposal = utils::proto::unmarshal(&signed_proposal.proposal_bytes)?;
        let header = proposal.header.clone().ok_or_else(||from_str("header is null"))?;

        self.verify(&signed_proposal, &header.creator)?;

        let payload: ContractProposalPayload = utils::proto::unmarshal(&proposal.payload)?;
        let contract = payload.contract_id.ok_or_else(||from_str("contract id is null"))?;
        if payload.input.is_none() {
            return Err(from_str("input is null"));
        }

        let tx_simulator = self
            .support
            .get_transaction_simulator(&header.channel_id, &header.tx_id)?
            .ok_or_else(||from_str("simulator not found"))?;

        let tx_params = TransactionParams {
            tx_id: header.tx_id.clone(),
            channel_id: header.channel_id.clone(),
            namespace: contract.name.clone(),
            // signed_proposal,
            proposal,
            tx_simulator,
        };

        let (rw_set, response, event) = self.simulate_proposal(tx_params, &contract.name).await?;

        if response.status != 200 {
            return Err(from_str(&response.message));
        }

        let event_bytes = match event {
            Some(e) => utils::proto::marshal(&e).map_err(|e| into_status(Box::new(e)))?,
            None => vec![],
        };
        let payload = ProposalResponsePayload {
            results: utils::proto::marshal(&rw_set)?,
            events: event_bytes,
        };
        let proposal_response = ProposalResponse {
            version: 0,
            timestamp: None,
            response: Some(response),
            payload: utils::proto::marshal(&payload)?,
            endorsement: None,
        };

        if header.header_type == HeaderType::Invoke as i32 {
            let tx = Transaction {
                signed_proposal: Some(signed_proposal),
                response: vec![proposal_response.clone()],
            };
            self.support.broadcast(&tx).await?;
        }

        info!("tx_id:{:?} -> {:?}", header.tx_id, proposal_response);
        Ok(proposal_response)
    }

    async fn simulate_proposal(
        &self,
        tx_params: TransactionParams,
        contract: &str,
    ) -> Result<(TxReadWriteSet, Response, Option<ContractEvent>)> {
        let (resp, event) = self.support.execute(&tx_params, &contract).await?;

        let rw_set = tx_params.tx_simulator.get_tx_simulation_results()?;

        Ok((rw_set, resp, event))
    }
}

#[async_trait::async_trait]
impl<S: ISupport> Endorser for Server<S> {
    async fn process_proposal(&self, request: Request<BatchSubmit>) -> RpcResult<BatchResponse> {
        let mut batch = request.into_inner();

        let mut bret = BatchResponse {
            proposal_response_list: Vec::with_capacity(batch.signed_proposal_list.len()),
        };

        let len = batch.signed_proposal_list.len();
        for index in 0..len {
            let signed_prop = batch.signed_proposal_list.remove(0);
            let proposal_response = match self.process(signed_prop).await {
                Ok(r) => r,
                Err(e) => ProposalResponse {
                    version: 0,
                    timestamp: None,
                    response: Some(Response {
                        status: 500,
                        message: e.to_string(),
                        payload: vec![],
                    }),
                    payload: vec![],
                    endorsement: None,
                },
            };
            bret.proposal_response_list.insert(index, proposal_response)
        }

        into_rpc_response(bret)
    }

    async fn ping(&self, _request: Request<Empty>) -> RpcResult<Empty> {
        into_rpc_response(Empty {})
    }
}
