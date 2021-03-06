use error::*;
use silk_proto::*;

// Height represents the height of a transaction in blockchain
#[derive(Clone, Debug, Eq, PartialEq, Copy)]
pub struct Height {
    pub block_num: u64,
    pub tx_num: u64,
}

impl Height {
    pub fn new(block_num: u64, tx_num: u64) -> Self {
        Height { block_num, tx_num }
    }

    pub fn new_from_bytes(b: &[u8]) -> Result<Self> {
        let h = utils::proto::unmarshal::<VersionedHeight>(b)?;
        Ok(Height {
            block_num: h.block_num,
            tx_num: h.tx_num,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        utils::proto::marshal(&VersionedHeight {
            block_num: self.block_num,
            tx_num: self.tx_num,
        })
        .unwrap_or_default()
    }

    // Compare return a -1, zero, or +1 based on whether this height is
    // less than, equals to, or greater than the specified height respectively.
    pub fn compare(&self, h: Height) -> i32 {
        let res = if self.block_num != h.block_num {
            (self.block_num - h.block_num) as i32
        } else if self.tx_num != h.tx_num {
            (self.tx_num - h.tx_num) as i32
        } else {
            0
        };

        match res {
            x if x > 0 => 1,
            x if x < 0 => -1,
            _ => 0,
        }
    }
}

impl From<Version> for Height {
    fn from(ver: Version) -> Self {
        Height {
            block_num: ver.block_num,
            tx_num: ver.tx_num,
        }
    }
}

// are_same returns true if both the heights are either nil or equal
pub fn are_same(h1: Option<Height>, h2: Option<Height>) -> bool {
    if h1.is_none() {
        return h2.is_none();
    }
    if h2.is_none() {
        return false;
    }

    h1.unwrap().compare(h2.unwrap()) == 0
}

#[cfg(test)]
mod tests {
    use crate::statedb::{are_same, Height};

    #[test]
    fn it_works() {
        let b = are_same(
            Some(Height {
                block_num: 1,
                tx_num: 0,
            }),
            Some(Height {
                block_num: 1,
                tx_num: 0,
            }),
        );
        assert_eq!(b, true);
    }
}
