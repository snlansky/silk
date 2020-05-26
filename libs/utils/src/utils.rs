use error::*;
use silk_proto::{Block, Transaction, Proposal, Header};
use crate::proto;

pub fn get_chain_id_from_block(_block: &Block) -> Result<String> {
    unimplemented!()
}

pub fn get_tx_header_from_data(data: &[u8]) -> Result<(Transaction, Header)> {
    let tx = proto::unmarshal::<Transaction>(&data)?;
    let signed_proposal = tx.signed_proposal.clone().ok_or(from_str("transaction signed proposal is null"))?;
    let proposal = proto::unmarshal::<Proposal>(&signed_proposal.proposal_bytes)?;
    let tx_header = proposal.header.ok_or(from_str("transaction header is null"))?;
    Ok((tx, tx_header))
}
