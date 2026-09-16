//! Tests for the bitfield.
//!
//! The bit-order tests are the important ones. `BitTorrent` numbers bits from
//! the left of each byte, which is the reverse of how integers are laid out.
//! Getting it backwards produces a bitfield that works consistently with
//! itself but disagrees with every peer — and the only symptom is pieces
//! failing their hash check for no obvious reason.

use super::*;

// --- bit order ---------------------------------------------------------

/// Piece 0 is the *highest* bit of the first byte, so the byte reads 128.
/// If this gives 1, the `7 - slot` flip is missing.
#[test]
fn piece_zero_is_the_leftmost_bit() {
    let mut field = Bitfield::new(8);
    field.set(0);

    assert_eq!(field.bits(), &[0b1000_0000]);
}

/// Piece 7 is the lowest bit of the first byte.
#[test]
fn piece_seven_is_the_rightmost_bit() {
    let mut field = Bitfield::new(8);
    field.set(7);

    assert_eq!(field.bits(), &[0b0000_0001]);
}

/// Piece 8 starts the second byte.
#[test]
fn piece_eight_moves_to_the_next_byte() {
    let mut field = Bitfield::new(16);
    field.set(8);

    assert_eq!(field.bits(), &[0b0000_0000, 0b1000_0000]);
}

#[test]
fn bits_land_in_order() {
    let mut field = Bitfield::new(8);
    field.set(1);
    field.set(2);
    field.set(4);

    assert_eq!(field.bits(), &[0b0110_1000]);
}

// --- allocation --------------------------------------------------------

/// One *bit* per piece, not one byte — 20 pieces need 3 bytes.
#[test]
fn allocates_by_bits_not_bytes() {
    assert_eq!(Bitfield::new(20).bits().len(), 3);
    assert_eq!(Bitfield::new(8).bits().len(), 1);
    assert_eq!(Bitfield::new(9).bits().len(), 2);
    assert_eq!(Bitfield::new(0).bits().len(), 0);
}

#[test]
fn starts_empty() {
    let field = Bitfield::new(20);

    assert_eq!(field.count(), 0);
    assert!(!field.has(0));
    assert!(!field.is_complete());
}

// --- get and set -------------------------------------------------------

#[test]
fn set_then_has() {
    let mut field = Bitfield::new(20);
    field.set(5);

    assert!(field.has(5));
    assert!(!field.has(4));
    assert!(!field.has(6));
}

#[test]
fn setting_twice_is_harmless() {
    let mut field = Bitfield::new(20);
    field.set(5);
    field.set(5);

    assert_eq!(field.count(), 1);
}

/// Piece numbers come from peers, so they can be anything. Out of range must
/// answer "no" rather than panic or read a neighbouring byte.
#[test]
fn out_of_range_is_false() {
    let field = Bitfield::new(20);

    assert!(!field.has(20));
    assert!(!field.has(1_000_000));
}

/// A piece number inside the spare bits of the last byte still counts as out
/// of range: 20 pieces occupy 3 bytes, so byte 2 exists and piece 21 would
/// land in it.
#[test]
fn spare_bit_positions_are_out_of_range() {
    let mut field = Bitfield::new(20);
    field.set(21);

    assert!(!field.has(21));
    assert_eq!(field.count(), 0);
}

// --- counting ----------------------------------------------------------

#[test]
fn counts_set_pieces() {
    let mut field = Bitfield::new(20);
    field.set(0);
    field.set(9);
    field.set(19);

    assert_eq!(field.count(), 3);
}

#[test]
fn complete_when_every_piece_is_set() {
    let mut field = Bitfield::new(12);

    for piece in 0..12 {
        field.set(piece);
    }

    assert_eq!(field.count(), 12);
    assert!(field.is_complete());
}

// --- from a peer -------------------------------------------------------

#[test]
fn accepts_a_valid_bitfield() {
    // 12 pieces, 2 bytes. Pieces 0 and 8 set; the last 4 bits are spare.
    let field = Bitfield::from_bytes(&[0b1000_0000, 0b1000_0000], 12).unwrap();

    assert!(field.has(0));
    assert!(field.has(8));
    assert_eq!(field.count(), 2);
}

#[test]
fn accepts_an_exact_multiple_of_eight() {
    // No spare bits at all, so the spare-bit check must not misfire.
    let field = Bitfield::from_bytes(&[0b1111_1111], 8).unwrap();

    assert_eq!(field.count(), 8);
    assert!(field.is_complete());
}

#[test]
fn rejects_too_few_bytes() {
    assert!(Bitfield::from_bytes(&[0b0000_0000], 20).is_err());
}

#[test]
fn rejects_too_many_bytes() {
    assert!(Bitfield::from_bytes(&[0, 0, 0, 0], 20).is_err());
}

/// A peer setting bits past the last real piece is misbehaving. Accepting it
/// would let `count` report more pieces than the torrent has.
#[test]
fn rejects_set_spare_bits() {
    // 12 pieces: 4 real bits in the second byte, 4 spare. The lowest bit here
    // is spare and must not be set.
    let result = Bitfield::from_bytes(&[0b0000_0000, 0b0000_0001], 12);

    assert!(result.is_err());
}

#[test]
fn accepts_an_empty_bitfield() {
    let field = Bitfield::from_bytes(&[], 0).unwrap();

    assert_eq!(field.count(), 0);
    // A torrent with no pieces is vacuously complete.
    assert!(field.is_complete());
}

// --- round trip --------------------------------------------------------

/// What a peer sends must come back out unchanged, since you forward your
/// own bitfield the same way.
#[test]
fn bytes_survive_a_round_trip() {
    let original = [0b1010_1010, 0b1100_0000];
    let field = Bitfield::from_bytes(&original, 10).unwrap();

    assert_eq!(field.bits(), &original);
}
