use error::*;
use silk_proto::p2p_server::P2p;
use silk_proto::*;

use tonic::{Request, Streaming};

#[derive(Clone)]
pub struct Server {}

#[async_trait::async_trait]
impl P2p for Server {
    type BroadcastStream = Receiver<Message>; // mpsc::Receiver<Result<Message, Status>>;

    async fn broadcast(
        &self,
        _request: Request<Streaming<Message>>,
    ) -> RpcResult<Self::BroadcastStream> {
        unimplemented!()
    }
}
