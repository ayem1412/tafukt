use crate::protocol::decoder::error::DecoderError;

fn bytes_to_string(bytes: Vec<u8>) -> String {
    bytes.iter().map(|&byte| byte as char).collect::<String>()
}

pub fn bytes_to_integer(bytes: Vec<u8>) -> Result<i64, DecoderError> {
    match bytes_to_string(bytes).parse::<i64>() {
        Ok(integer) => Ok(integer),
        Err(_) => return Err(DecoderError::InvalidIntegerSyntax),
    }
}
