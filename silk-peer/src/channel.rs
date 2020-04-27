use std::collections::HashMap;

#[derive(Clone)]
pub struct Channel {
    name: String,
    storage: HashMap<String, Vec<u8>>,
}

impl Channel {
    pub fn new(name: String) -> Self {
        Channel {
            name,
            storage: HashMap::new(),
        }
    }
}
