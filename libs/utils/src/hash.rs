use sha2::digest::DynDigest;
use sha2::{Digest, Sha256};
use std::net::SocketAddr;

pub fn parse_addr(addr: SocketAddr) -> (String, u16) {
    (addr.ip().to_string(), addr.port())
}

pub fn compute_sha256(data: &[u8]) -> Box<[u8]> {
    let mut hasher = Sha256::new();
    <Sha256 as DynDigest>::input(&mut hasher, data);
    <Sha256 as DynDigest>::result_reset(&mut hasher)
}

pub fn compute_vec_sha256(data: &Vec<Vec<u8>>) -> Box<[u8]> {
    let mut hasher = Sha256::new();
    for d in data {
        <Sha256 as DynDigest>::input(&mut hasher, d);
    }
    <Sha256 as DynDigest>::result_reset(&mut hasher)
}

pub fn hex_to_string(data: &[u8]) -> String {
    hex::encode(data)
}

#[cfg(test)]
mod tests {
    use crate::hash::{compute_sha256, hex_to_string, parse_addr, compute_vec_sha256};

    #[macro_use]
    #[test]
    fn test_parse_addr() {
        let (host, port) = parse_addr("127.0.0.1:8090".parse().unwrap());
        assert_eq!(host, "127.0.0.1".to_string());
        assert_eq!(port, 8090);
    }

    #[test]
    fn test_hash() {
        let h = compute_sha256("hello world".as_bytes());
        let res = hex!("b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
        assert_eq!(h[..], res[..])
    }

    #[test]
    fn test_hex_to_string() {
        let data = vec![
            185, 77, 39, 185, 147, 77, 62, 8, 165, 46, 82, 215, 218, 125, 171, 250, 196, 132, 239,
            227, 122, 83, 128, 238, 144, 136, 247, 172, 226, 239, 205, 233,
        ];
        let data = data.into_iter().map(|i| i as u8).collect::<Vec<u8>>();

        let res = hex_to_string(data.as_slice());
        assert_eq!(
            res,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9".to_string()
        )
    }

    #[test]
    fn test_vec_hash() {
        let data = vec![vec![185, 77, 39, 185], vec![82, 215, 218, 125, 171], vec![144, 136, 247, 172, 226, 239, 205, 233]];
        let hash = compute_vec_sha256(&data);
        println!("{:?}", hash)
    }
}
