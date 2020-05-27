use prost_types::Timestamp;
use chrono::prelude::*;

pub fn timestamp() -> Timestamp {
    let dt = Local::now();
    Timestamp{ seconds: dt.timestamp(), nanos: dt.timestamp_subsec_nanos() as i32 }
}

#[cfg(test)]
mod tests {
    use crate::time::timestamp;

    # [test]
    fn test_timestamp() {
        let ts = timestamp();
        println!("{:?}", ts);
    }
}
