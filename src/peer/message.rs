use bytes::{Buf, BufMut, Bytes, BytesMut};

/// 0 - choke
/// 1 - unchoke
/// 2 - interested
/// 3 - not interested
/// 4 - have
/// 5 - bitfield
/// 6 - request
/// 7 - piece
/// 8 - cancel
#[derive(Debug)]
pub enum Message {
    KeepAlive,
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    // Have,
    Bitfield(Vec<u8>),
    Request { index: u32, begin: u32, length: u32 },
    Piece { index: u32, begin: u32, data: Bytes },
    // Cancel,
}

impl Message {
    pub fn encode(&self) -> Bytes {
        let mut buf = BytesMut::new();

        match self {
            Message::KeepAlive => buf.put_u32(0),
            Message::Choke => {
                buf.put_u32(1);
                buf.put_u8(0);
            },
            Message::Unchoke => {
                buf.put_u32(1);
                buf.put_u8(1);
            },
            Message::Interested => {
                buf.put_u32(1);
                buf.put_u8(2);
            },
            Message::NotInterested => {
                buf.put_u32(1);
                buf.put_u8(3);
            },
            Message::Bitfield(data) => {
                buf.put_u32(1 + data.len() as u32);
                buf.put_u8(5);
                buf.extend_from_slice(data);
            },
            Message::Request { index, begin, length } => {
                buf.put_u32(19);
                buf.put_u8(6);
                buf.put_u32(*index);
                buf.put_u32(*begin);
                buf.put_u32(*length);
            },
            Message::Piece { index, begin, data } => {
                buf.put_u32(9 + data.len() as u32);
                buf.put_u8(7);
                buf.put_u32(*index);
                buf.put_u32(*begin);
                buf.extend_from_slice(data);
            },
        }

        buf.freeze()
    }

    pub fn decode(buf: &mut BytesMut) -> anyhow::Result<Option<Self>> {
        if buf.len() < 4 {
            return Ok(None);
        }

        let length = u32::from_be_bytes(buf[..4].try_into().unwrap()) as usize;
        if buf.len() < 4 + length {
            return Ok(None);
        }
        buf.advance(4);

        if length == 0 {
            return Ok(Some(Message::KeepAlive));
        }

        let message = match buf.get_u8() {
            0 => Message::Choke,
            1 => Message::Unchoke,
            2 => Message::Interested,
            3 => Message::NotInterested,
            5 => Message::Bitfield(buf.split_to(length - 1).to_vec()),
            6 => Message::Request { index: buf.get_u32(), begin: buf.get_u32(), length: buf.get_u32() },
            7 => Message::Piece { index: buf.get_u32(), begin: buf.get_u32(), data: buf.split_to(length - 9).freeze() },
            _ => {
                buf.advance(length - 1);
                return Ok(None);
            },
        };

        Ok(Some(message))
    }
}
