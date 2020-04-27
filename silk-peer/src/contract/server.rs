use silk_proto::*;

use super::*;
use futures::StreamExt;
use silk_proto::message::MessageType;
use tokio::sync::mpsc;
use tonic::{Code, Request, Response, Status, Streaming};

#[derive(Clone)]
pub struct Server<S: IContractSupport> {
    support: S,
}

impl<S: IContractSupport> Server<S> {
    pub fn new(support: S) -> Self {
        Server { support }
    }
}

#[async_trait::async_trait]
impl<S: IContractSupport> contract_server::Contract for Server<S> {
    type RegisterStream = Receiver<Message>; //mpsc::Receiver<Result<Message, Status>>;
                                             // Pin<Box<dyn Stream<Item = Result<Message, Status>> + Send + Sync>>;

    async fn register(
        &self,
        request: Request<Streaming<Message>>,
    ) -> RpcResult<Self::RegisterStream> {
        let (tx, rx) = mpsc::channel(1000);

        let mut stream = request.into_inner();

        let handler = match stream.next().await.unwrap() {
            Ok(msg) => match msg.message_type {
                t if t == MessageType::ContractRegister as i32 => {
                    let register: ContractRegister = utils::proto::unmarshal(&msg.content)
                        .map_err(|e| into_status(Box::new(e)))?;
                    info!("start register contract [{:?}]", register);
                    let handler = Contract::new(register.clone(), tx);
                    self.support
                        .register(handler.clone())
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
                        error!(
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
