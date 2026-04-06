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
/// 'choke', 'unchoke', 'interested', and 'not interested' have no payload.
pub enum Message {
    KeepAlive,
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    // Have,
    /// 'bitfield' is only ever sent as the first message.
    /// Its payload is a bitfield with each index that downloader has sent set to one and the rest
    /// set to zero. Downloaders which don't have anything yet may skip the 'bitfield' message.
    /// The first byte of the bitfield corresponds to indices 0 - 7 from high bit to low bit,
    /// respectively. The next one 8-15, etc. Spare bits at the end are set to zero.
    Bitfield(Vec<u8>),

    /// 'request' messages contain an index, begin, and length. The last two are byte offsets.
    /// Length is generally a power of two unless it gets truncated by the end of the file.
    /// All current implementations use 2^14 (16 kiB), and close connections which request an amount
    /// greater than that.
    Request {
        index: u32,
        begin: u32,
        length: u32,
    },

    /// 'piece' messages contain an index, begin, and piece.
    /// Note that they are correlated with request messages implicitly.
    /// It's possible for an unexpected piece to arrive if choke and unchoke messages are sent in
    /// quick succession and/or transfer is going very slowly.
    Piece {
        index: u32,
        begin: u32,
        data: Bytes,
    },
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

    pub fn decode(buf: &mut BytesMut) -> Option<Self> {
        if buf.len() < 4 {
            return None;
        }

        let length = u32::from_be_bytes(buf[..4].try_into().unwrap()) as usize;
        if buf.len() < 4 + length {
            return None;
        }
        buf.advance(4);

        if length == 0 {
            return Some(Message::KeepAlive);
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
                return None;
            },
        };

        Some(message)
    }
}
