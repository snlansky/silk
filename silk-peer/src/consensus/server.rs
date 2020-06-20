use silk_proto::*;

use super::*;
use futures::StreamExt;
use tokio::sync::mpsc;
use tonic::{Code, Request, Response, Status, Streaming};

#[derive(Clone)]
pub struct Server<S: IConsensusSupport> {
    support: S,
}

impl<S: IConsensusSupport> Server<S> {
    pub fn new(support: S) -> Self {
        Server { support }
    }
}

#[async_trait::async_trait]
impl<S: IConsensusSupport> consensus_server::Consensus for Server<S> {
    type RegisterStream = Receiver<Message>; //mpsc::Receiver<Result<Message, Status>>;
                                             // Pin<Box<dyn Stream<Item = Result<Message, Status>> + Send + Sync>>;

    async fn register(
        &self,
        request: Request<Streaming<Message>>,
    ) -> RpcResult<Self::RegisterStream> {
        let (tx, rx) = mpsc::channel(32);

        let mut stream = request.into_inner();

        let handler = match stream.next().await.unwrap() {
            Ok(msg) => match msg.message_type {
                t if t == message::MessageType::ConsensusRegister as i32 => {
                    let register: ConsensusRegister =
                        utils::proto::unmarshal(msg.content.as_slice()).unwrap();
                    info!("start register contract [{:?}]", register);
                    let handler = Consensus::new(register.clone(), tx);
                    // TODO: register consensus to consensus_support
                    self.support
                        .register(handler.clone())
                        .await
                        .map_err(into_status)?;
                    handler
                }
                _ => {
                    let describe = format!("unhandled massage type {:?}", msg.message_type);
                    return Err(Status::new(Code::InvalidArgument, describe));
                }
            },
            Err(s) => return Err(s),
        };

        self.support.update_chain().await.map_err(into_status)?;

        tokio::spawn(async move {
            while let Some(res) = stream.next().await {
                match res {
                    Ok(msg) => {
                        // tx.send(Ok(msg)).await;
                        if handler.handler(msg).await.is_err() {
                            error!("will close client");
                            break;
                        }
                    }
                    Err(stat) => {
                        println!(
                            "client closed reason {:?} {:?}",
                            stat.code(),
                            stat.message()
                        );
                        break;
                    }
                }
            }
        });

        Ok(Response::new(rx))
    }
}
