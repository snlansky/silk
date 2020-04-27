use error::*;
use silk_proto::message::MessageType;
use silk_proto::*;

pub trait Handler: Sync + Send + 'static {
    fn handler(&self, message: Message) -> Result<()>;
}

#[derive(Clone)]
pub struct EventHandler {
    support: Option<i32>,
}

impl EventHandler {
    pub fn new() -> Self {
        EventHandler { support: None }
    }

    pub fn set_support(&mut self, support: i32) {
        self.support = Some(support);
    }
}

impl Handler for EventHandler {
    fn handler(&self, msg: Message) -> Result<()> {
        debug!("received consensus message: {:?}", msg);
        match msg.message_type {
            t if t == MessageType::ConsensusNotifyBlockCommit as i32 => {
                let _block = utils::proto::unmarshal::<Block>(&msg.content)?;
                Ok(())
            }

            _ => {
                let describe = format!("unhandled massage type {:?}", msg.message_type);
                Err(from_str(&describe))
            }
        }
    }
}
