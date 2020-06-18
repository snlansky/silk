use rand::{thread_rng, RngCore};

// NonceSize is the default NonceSize
const NONCE_SIZE: usize = 24;

// get_random_nonce returns a random byte array of length NonceSize
pub fn get_random_nonce() -> Vec<u8>  {
    let mut buf = [0u8; NONCE_SIZE];
    let mut rng = thread_rng();
    rng.fill_bytes(&mut buf);
    buf.to_vec()
}

#[cfg(test)]
mod tests {
    use crate::random::get_random_nonce;

    # [test]
    fn test_random_nonce() {
        let nonce = get_random_nonce();
        println!("{:?}", nonce);
    }
}
