use contract::{start, Contract, ContractStub};
use silk_proto::Response;

struct ContractSimple;

impl Contract for ContractSimple {
    fn init<'a>(&self, _stub: &'a mut ContractStub) -> Response {
        unimplemented!()
    }

    fn invoke<'a>(&self, _stub: &'a mut ContractStub) -> Response {
        Response {
            status: 200,
            message: "ok".to_string(),
            payload: "so easy".as_bytes().to_vec(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n*** start contract-simple ***");
    start("contract-simple".to_string(), Box::new(ContractSimple)).await?;
    Ok(())
}
