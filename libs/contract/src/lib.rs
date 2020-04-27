mod stub;
pub use stub::*;

mod shim;
pub use shim::*;
use silk_proto::*;

pub trait Contract {
    fn init<'a>(&self, stub: &'a mut ContractStub) -> Response;
    fn invoke<'a>(&self, stub: &'a mut ContractStub) -> Response;
}

#[repr(i32)]
pub enum Status {
    Success = 200,
    Failed = 500,
}

pub fn response_success(payload: Vec<u8>) -> Response {
    Response {
        status: Status::Success as i32,
        message: "ok".to_string(),
        payload,
    }
}

pub fn response_failed(info: String) -> Response {
    Response {
        status: Status::Failed as i32,
        message: info,
        payload: vec![],
    }
}

// pub trait IContractStub {
//     fn get_args(&self) -> Vec<Vec<u8>>;
//     fn get_txid(&self) -> String;
//     fn get_channel_id(&self) -> String;
//     fn get_address(&self) -> Result<String>;
//     async fn get_state(&mut self, key :&String) -> Result<Vec<u8>>;
//     fn put_state(&mut self, key: String, value: &[u8]) -> Result<()>;
//     fn del_state(&mut self, key: &String) -> Result<()>;
//     fn get_time(&self) -> Result<Timestamp>;
//     fn set_event(&mut self, name :String, payload: &[u8]) -> Result<()>;
// }

#[cfg(test)]
mod tests {
    use crate::Status;

    #[test]
    fn it_works() {
        assert_eq!(Status::Success as i32, 200);
    }
}
