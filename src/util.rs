use std::io;

pub fn invalid_data_error(msg: &str) -> Result<(), io::Error> {
    Err(io::Error::new(io::ErrorKind::InvalidData, msg))
}
