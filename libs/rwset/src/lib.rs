mod builder;
mod rwset;
mod validate;
mod key;

#[macro_use]
extern crate log;

pub use builder::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
