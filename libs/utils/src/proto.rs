use prost::{DecodeError, EncodeError, Message};
use byteorder::{WriteBytesExt, BigEndian};


pub fn marshal<T: Message>(msg: &T) -> Result<Vec<u8>, EncodeError> {
    let mut bytes = Vec::with_capacity(msg.encoded_len());
    msg.encode(&mut bytes)?;
    Ok(bytes)
}

pub fn unmarshal<T: Message + Default>(buf: &[u8]) -> Result<T, DecodeError> {
    let mut t = T::default();
    t.merge(buf)?;
    Ok(t)
}

pub fn marshal_with_length<T: Message>(msg: &T) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let len = msg.encoded_len();
    let mut bytes = Vec::with_capacity(len + 4);
    bytes.write_u32::<BigEndian>(len as u32)?;

    msg.encode(&mut bytes)?;
    Ok(bytes)
}


#[cfg(test)]
mod tests {
    use crate::proto::{marshal, unmarshal, marshal_with_length};
    use silk_proto::*;
    use std::collections::HashMap;

    #[test]
    fn test() {
        let reg = ContractRegister {
            name: "kv".to_string(),
            decorations: Default::default(),
        };
        let ret = marshal(&reg);
        assert!(ret.is_ok());

        let new = unmarshal::<ContractRegister>(&ret.unwrap());
        assert!(new.is_ok());

        assert_eq!(reg, new.unwrap());
    }

    #[test]
    fn test_length() {
        let mut dec = HashMap::new();
        dec.insert("id".to_string(), vec![8, 12]);
        let reg = ContractRegister {
            name: "contract name".to_string(),
            decorations: dec,
        };
        let ret = marshal(&reg).unwrap();

        let ret_len = marshal_with_length(&reg).unwrap();

        println!("{:?}", ret);
        println!("{:?}", ret_len);

        assert_eq!(ret.len()  + 4, ret_len.len());
        assert_eq!(ret.len(), 25);
    }
}
