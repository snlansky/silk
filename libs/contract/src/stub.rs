use error::*;
use silk_proto::contract_client::ContractClient;
use silk_proto::*;
use tonic::transport::Channel;

pub struct ContractStub<'a> {
    client: &'a ContractClient<Channel>,
    proposal: Proposal,
    event: Option<ContractEvent>,
}

impl<'a> ContractStub<'a> {
    pub fn new(client: &'a ContractClient<Channel>, proposal: Proposal) -> Self {
        ContractStub {
            client,
            proposal,
            event: None,
        }
    }

    fn get_args(&self) -> Vec<Vec<u8>> {
        unimplemented!()
    }

    fn get_txid(&self) -> String {
        unimplemented!()
    }

    fn get_channel_id(&self) -> String {
        unimplemented!()
    }

    fn get_address(&self) -> Result<String> {
        unimplemented!()
    }

    async fn get_state(&mut self, _key: &String) -> Result<Vec<u8>> {
        unimplemented!()
    }

    fn put_state(&mut self, _key: String, _value: &[u8]) -> Result<()> {
        unimplemented!()
    }

    fn del_state(&mut self, _key: &String) -> Result<()> {
        unimplemented!()
    }

    fn get_time(&self) -> Result<prost_types::Timestamp> {
        unimplemented!()
    }

    fn set_event(&mut self, _name: String, _payload: &[u8]) -> Result<()> {
        unimplemented!()
    }

    pub fn get_event(&self) -> Option<ContractEvent> {
        self.event.clone()
    }
}
