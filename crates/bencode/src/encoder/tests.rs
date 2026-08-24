use super::*;

fn enc(value: &Bencode) -> Vec<u8> {
    encode(value)
}

#[test]
fn integer_positive() {
    assert_eq!(enc(&Bencode::Integer(42)), b"i42e");
}

#[test]
fn integer_negative() {
    assert_eq!(enc(&Bencode::Integer(-42)), b"i-42e");
}

#[test]
fn integer_zero() {
    assert_eq!(enc(&Bencode::Integer(0)), b"i0e");
}

#[test]
fn integer_extremes() {
    assert_eq!(enc(&Bencode::Integer(i64::MAX)), b"i9223372036854775807e");
    assert_eq!(enc(&Bencode::Integer(i64::MIN)), b"i-9223372036854775808e");
}

#[test]
fn string_normal() {
    assert_eq!(enc(&Bencode::String(b"spam")), b"4:spam");
}

#[test]
fn string_empty() {
    assert_eq!(enc(&Bencode::String(b"")), b"0:");
}

#[test]
fn string_arbitrary_bytes() {
    assert_eq!(
        enc(&Bencode::String(&[0xff, 0x00, 0xfe])),
        b"3:\xff\x00\xfe"
    );
}

#[test]
fn list_empty() {
    assert_eq!(enc(&Bencode::List(vec![])), b"le");
}

#[test]
fn list_with_items() {
    let value = Bencode::List(vec![Bencode::String(b"spam"), Bencode::Integer(42)]);
    assert_eq!(enc(&value), b"l4:spami42ee");
}

#[test]
fn dictionary_empty() {
    assert_eq!(enc(&Bencode::Dictionary(BTreeMap::new())), b"de");
}

/// Keys must come out sorted regardless of insertion order.
#[test]
fn dictionary_sorts_keys() {
    let mut map = BTreeMap::new();
    map.insert(b"spam".as_slice(), Bencode::String(b"eggs"));
    map.insert(b"cow".as_slice(), Bencode::String(b"moo"));

    assert_eq!(enc(&Bencode::Dictionary(map)), b"d3:cow3:moo4:spam4:eggse");
}

#[test]
fn nested() {
    let inner = Bencode::List(vec![Bencode::String(b"a"), Bencode::String(b"b")]);
    let mut map = BTreeMap::new();
    map.insert(b"list".as_slice(), inner);

    assert_eq!(enc(&Bencode::Dictionary(map)), b"d4:listl1:a1:bee");
}
