use std::error::Error;

use tonic::transport::Channel;

use silk_proto::*;

use silk_proto::endorser_client::EndorserClient;

async fn commit_transaction(client: &mut EndorserClient<Channel>) -> Result<(), Box<dyn Error>> {
    let payload = ContractProposalPayload {
        contract_id: Some(ContractId {
            name: "contract-simple".to_string(),
        }),
        input: Some(ContractInput {
            args: vec![],
            decorations: Default::default(),
            is_init: false,
        }),
        transient_map: Default::default(),
        timeout: 0,
    };

    let proposal = Proposal {
        header: Some(Header {
            header_type: HeaderType::Invoke as i32,
            version: 0,
            timestamp: None,
            channel_id: "system_channel".to_string(),
            tx_id: "test_tx_id".to_string(),
            tls_cert_hash: vec![],
            creator: vec![],
            nonce: vec![],
        }),
        payload: utils::proto::marshal(&payload)?,
    };
    let sp = SignedProposal {
        proposal_bytes: utils::proto::marshal(&proposal)?,
        signature: vec![],
    };

    let batch = BatchSubmit {
        signed_proposal_list: vec![sp],
    };
    let resp = client.process_proposal(tonic::Request::new(batch)).await?;
    for r in resp.into_inner().proposal_response_list {
        let response = r.response.unwrap();
        println!(
            "RETURN: {:?}, {:?}, {:?}",
            response.status,
            String::from_utf8(response.payload),
            response.message,
        );
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = EndorserClient::connect("http://127.0.0.1:8080").await?;

    println!("\n*** BIDIRECTIONAL STREAMING ***");
    commit_transaction(&mut client).await?;

    Ok(())
}
