use tonic::{Response, Status};

use tokio::sync::mpsc;

pub type Sender<T> = mpsc::Sender<std::result::Result<T, Status>>;

pub type Receiver<T> = mpsc::Receiver<std::result::Result<T, Status>>;

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

pub type Result<T> = std::result::Result<T, Error>;

pub type RpcResult<T> = std::result::Result<Response<T>, Status>;

pub fn into_rpc_response<T>(t: T) -> RpcResult<T> {
    Ok(tonic::Response::new(t))
}

pub fn into_status(err: Error) -> Status {
    tonic::Status::unknown(err.to_string())
}

pub fn from_str(info: &str) -> Error {
    Error::from(info.to_string())
}
