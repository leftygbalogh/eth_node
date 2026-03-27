//! Ethereum primitive types — Address, H256/B256, U256, ABI encode/decode, RLP encode/decode.
//! Spec ref: FORMAL_SPEC.md §4 FR-001

use std::str::FromStr;

use alloy_primitives::{Address, B256, U256};
use alloy_sol_types::{sol, SolValue};
use alloy_rlp::{Decodable, Encodable, RlpDecodable, RlpEncodable};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error, PartialEq)]
pub enum PrimitiveError {
    #[error("invalid hex: {0}")]
    InvalidHex(String),
    #[error("invalid length: expected {expected} bytes, got {actual}")]
    InvalidLength { expected: usize, actual: usize },
    #[error("ABI decode error: {0}")]
    AbiDecodeError(String),
    #[error("RLP decode error: {0}")]
    RlpDecodeError(String),
}

// ---------------------------------------------------------------------------
// Address helpers
// ---------------------------------------------------------------------------

/// Parse a hex string (with or without `0x`, checksummed or lowercase) into an `Address`.
pub fn parse_address(s: &str) -> Result<Address, PrimitiveError> {
    let trimmed = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")).unwrap_or(s);
    if trimmed.len() != 40 {
        return Err(PrimitiveError::InvalidLength {
            expected: 20,
            actual: trimmed.len() / 2,
        });
    }
    Address::from_str(s)
        .map_err(|e| PrimitiveError::InvalidHex(e.to_string()))
}

// ---------------------------------------------------------------------------
// ABI encode/decode
// ---------------------------------------------------------------------------

// Declare a minimal Solidity tuple type for ABI round-trip testing.
sol! {
    struct AbiTuple {
        uint256 a;
        address b;
        bool c;
    }
}

/// ABI-encode a `uint256`.
pub fn abi_encode_uint256(v: U256) -> Vec<u8> {
    v.abi_encode()
}

/// ABI-decode a `uint256` from a 32-byte word.
pub fn abi_decode_uint256(data: &[u8]) -> Result<U256, PrimitiveError> {
    U256::abi_decode(data, true)
        .map_err(|e| PrimitiveError::AbiDecodeError(e.to_string()))
}

/// ABI-encode an `Address`.
pub fn abi_encode_address(a: Address) -> Vec<u8> {
    a.abi_encode()
}

/// ABI-decode an `Address`.
pub fn abi_decode_address(data: &[u8]) -> Result<Address, PrimitiveError> {
    Address::abi_decode(data, true)
        .map_err(|e| PrimitiveError::AbiDecodeError(e.to_string()))
}

/// ABI-encode a `bytes32` value.
pub fn abi_encode_bytes32(b: B256) -> Vec<u8> {
    b.abi_encode()
}

/// ABI-decode a `bytes32` value.
pub fn abi_decode_bytes32(data: &[u8]) -> Result<B256, PrimitiveError> {
    B256::abi_decode(data, true)
        .map_err(|e| PrimitiveError::AbiDecodeError(e.to_string()))
}

/// ABI-encode a `bool`.
pub fn abi_encode_bool(b: bool) -> Vec<u8> {
    b.abi_encode()
}

/// ABI-decode a `bool`.
pub fn abi_decode_bool(data: &[u8]) -> Result<bool, PrimitiveError> {
    bool::abi_decode(data, true)
        .map_err(|e| PrimitiveError::AbiDecodeError(e.to_string()))
}

/// ABI-encode a Solidity `string`.
pub fn abi_encode_string(s: &str) -> Vec<u8> {
    s.abi_encode()
}

/// ABI-decode a Solidity `string`.
pub fn abi_decode_string(data: &[u8]) -> Result<String, PrimitiveError> {
    String::abi_decode(data, true)
        .map_err(|e| PrimitiveError::AbiDecodeError(e.to_string()))
}

/// ABI-encode an `AbiTuple` struct.
pub fn abi_encode_tuple(t: AbiTuple) -> Vec<u8> {
    t.abi_encode()
}

/// ABI-decode an `AbiTuple` struct.
pub fn abi_decode_tuple(data: &[u8]) -> Result<AbiTuple, PrimitiveError> {
    AbiTuple::abi_decode(data, true)
        .map_err(|e| PrimitiveError::AbiDecodeError(e.to_string()))
}

// ---------------------------------------------------------------------------
// RLP encode/decode
// ---------------------------------------------------------------------------

/// RLP-encode a `u64` integer.
pub fn rlp_encode_u64(v: u64) -> Vec<u8> {
    let mut buf = Vec::new();
    v.encode(&mut buf);
    buf
}

/// RLP-decode a `u64` integer.
pub fn rlp_decode_u64(data: &[u8]) -> Result<u64, PrimitiveError> {
    let mut buf = data;
    u64::decode(&mut buf).map_err(|e| PrimitiveError::RlpDecodeError(e.to_string()))
}

/// RLP-encode a byte slice.
pub fn rlp_encode_bytes(v: &[u8]) -> Vec<u8> {
    let mut buf = Vec::new();
    v.encode(&mut buf);
    buf
}

/// RLP-decode a byte vector.
/// alloy-rlp decodes `Vec<u8>` as a list; use `bytes::Bytes` to decode as a byte string.
pub fn rlp_decode_bytes(data: &[u8]) -> Result<Vec<u8>, PrimitiveError> {
    let mut buf = data;
    bytes::Bytes::decode(&mut buf)
        .map(|b| b.to_vec())
        .map_err(|e| PrimitiveError::RlpDecodeError(e.to_string()))
}

/// A simple RLP-encodable list wrapper for testing nested lists.
#[derive(Debug, PartialEq, RlpEncodable, RlpDecodable)]
pub struct RlpList {
    pub a: u64,
    pub b: Vec<u8>,
}

/// RLP-encode an `RlpList`.
pub fn rlp_encode_list(l: &RlpList) -> Vec<u8> {
    let mut buf = Vec::new();
    l.encode(&mut buf);
    buf
}

/// RLP-decode an `RlpList`.
pub fn rlp_decode_list(data: &[u8]) -> Result<RlpList, PrimitiveError> {
    let mut buf = data;
    RlpList::decode(&mut buf).map_err(|e| PrimitiveError::RlpDecodeError(e.to_string()))
}

// ── Event selector ──────────────────────────────────────────────────────────

/// Compute the topic-0 selector hash for an Ethereum event signature.
///
/// Equivalent to `keccak256(sig.as_bytes())` per the Solidity ABI specification.
/// If `sig` is already a `0x`-prefixed 32-byte hex hash it is returned as-is.
///
/// # Examples
/// ```
/// use eth_node::primitives::event_selector;
/// let t = event_selector("Transfer(address,address,uint256)");
/// assert_eq!(
///     format!("{t:?}"),
///     "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"
/// );
/// ```
pub fn event_selector(sig: &str) -> alloy_primitives::B256 {
    // Already a 32-byte hex hash? Pass through unchanged.
    if (sig.starts_with("0x") || sig.starts_with("0X")) && sig.len() == 66 {
        if let Ok(hash) = sig.parse::<alloy_primitives::B256>() {
            return hash;
        }
    }
    alloy_primitives::keccak256(sig.as_bytes())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{address, b256, uint};

    // --- Address parsing ---

    #[test]
    fn address_from_hex_with_prefix() {
        let a = parse_address("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap();
        assert_eq!(a, address!("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266"));
    }

    #[test]
    // alloy parses hex addresses case-insensitively: uppercase, lowercase, and EIP-55
    // checksummed (mixed-case) all decode to the same 20-byte value.
    fn address_from_hex_lowercase() {
        // lowercase is valid — just verify it parses without error
        assert!(parse_address("0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266").is_ok());
    }

    #[test]
    fn address_from_hex_uppercase() {
        // ALL-UPPERCASE hex is also valid — same address as the mixed-case EIP-55 form.
        let a = parse_address("0xF39FD6E51AAD88F6F4CE6AB8827279CFFFB92266").unwrap();
        assert_eq!(a, address!("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266"));
    }

    #[test]
    fn address_missing_prefix_still_parses() {
        let a = parse_address("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap();
        assert_eq!(a, address!("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266"));
    }

    #[test]
    fn address_wrong_length_returns_error() {
        let err = parse_address("0x1234").unwrap_err();
        assert!(matches!(err, PrimitiveError::InvalidLength { .. }));
    }

    #[test]
    fn address_invalid_hex_returns_error() {
        // 40 chars but not valid hex
        let err = parse_address("0xZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ").unwrap_err();
        assert!(matches!(
            err,
            PrimitiveError::InvalidHex(_) | PrimitiveError::InvalidLength { .. }
        ));
    }

    // --- U256 arithmetic ---

    #[test]
    fn u256_add_basic() {
        assert_eq!(uint!(1_U256) + uint!(2_U256), uint!(3_U256));
    }

    #[test]
    fn u256_checked_add_overflow() {
        assert!(U256::MAX.checked_add(uint!(1_U256)).is_none());
    }

    #[test]
    // U256 uses *wrapping* arithmetic for `+` — it never panics on overflow.
    // MAX + MAX wraps: 2×(2²⁵⁶−1) mod 2²⁵⁶ = 2²⁵⁶−2 = MAX−1.
    // `checked_add` is the safe alternative that explicitly signals overflow.
    fn u256_max_plus_max_wraps_and_checked_returns_none() {
        // Wrapping add: never panics, wraps mod 2^256.
        assert_eq!(U256::MAX + U256::MAX, U256::MAX - uint!(1_U256));
        // Checked add: returns None when overflow would occur.
        assert!(U256::MAX.checked_add(U256::MAX).is_none());
    }

    #[test]
    fn u256_checked_sub_underflow() {
        assert!(U256::ZERO.checked_sub(uint!(1_U256)).is_none());
    }

    // --- ABI uint256 ---

    #[test]
    fn abi_uint256_round_trip() {
        let v = uint!(42_U256);
        let enc = abi_encode_uint256(v);
        assert_eq!(enc.len(), 32);
        assert_eq!(abi_decode_uint256(&enc).unwrap(), v);
    }

    #[test]
    // uint256(1) is the simplest non-trivial known ABI vector: 31 zero bytes then 0x01.
    fn abi_uint256_known_vector() {
        let enc = abi_encode_uint256(uint!(1_U256));
        assert_eq!(enc.len(), 32);
        assert_eq!(&enc[..31], &[0u8; 31]);
        assert_eq!(enc[31], 1u8);
    }

    #[test]
    // uint256(0) encodes as 32 zero bytes — the minimal and most common boundary value.
    fn abi_uint256_zero_encodes_as_32_zero_bytes() {
        let enc = abi_encode_uint256(U256::ZERO);
        assert_eq!(enc, vec![0u8; 32]);
        assert_eq!(abi_decode_uint256(&enc).unwrap(), U256::ZERO);
    }

    #[test]
    // uint256(MAX) is the largest value that fits — all 32 bytes are 0xFF.
    fn abi_uint256_max_round_trip() {
        let enc = abi_encode_uint256(U256::MAX);
        assert_eq!(enc, vec![0xFFu8; 32]);
        assert_eq!(abi_decode_uint256(&enc).unwrap(), U256::MAX);
    }

    #[test]
    // A 31-byte slice is too short for a uint256 word — must return an error, never panic.
    fn abi_uint256_decode_rejects_short_slice() {
        let short = vec![0u8; 31];
        let err = abi_decode_uint256(&short).unwrap_err();
        assert!(matches!(err, PrimitiveError::AbiDecodeError(_)));
    }

    // --- ABI address ---

    #[test]
    fn abi_address_round_trip() {
        let a = address!("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266");
        let enc = abi_encode_address(a);
        assert_eq!(enc.len(), 32);
        assert_eq!(abi_decode_address(&enc).unwrap(), a);
    }

    // --- ABI bool ---

    #[test]
    fn abi_bool_true_round_trip() {
        let enc = abi_encode_bool(true);
        assert_eq!(enc.len(), 32);
        assert_eq!(enc[31], 1u8);
        assert!(abi_decode_bool(&enc).unwrap());
    }

    #[test]
    fn abi_bool_false_round_trip() {
        let enc = abi_encode_bool(false);
        assert_eq!(enc[31], 0u8);
        assert!(!abi_decode_bool(&enc).unwrap());
    }

    // --- ABI bytes32 ---

    #[test]
    fn abi_bytes32_round_trip() {
        let b = b256!("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f");
        let enc = abi_encode_bytes32(b);
        assert_eq!(enc.len(), 32);
        assert_eq!(abi_decode_bytes32(&enc).unwrap(), b);
    }

    // --- ABI string ---

    #[test]
    // ABI strings are raw UTF-8 bytes with no encoding-level length cap.
    // Any valid UTF-8 — ASCII, multi-byte Unicode, emoji, CJK, homoglyphs — round-trips
    // identically because the encoder stores raw bytes without transformation.
    fn abi_string_round_trip() {
        let s = "hello, ethereum";
        let enc = abi_encode_string(s);
        assert_eq!(abi_decode_string(&enc).unwrap(), s);
    }

    #[test]
    // Non-ASCII: Hungarian diacritics (ő is 2 bytes in UTF-8), CJK (世界 = 3 bytes each),
    // emoji (🦀 = 4 bytes), and homoglyphs (а Cyrillic ≠ a Latin) all encode as distinct
    // byte sequences and survive the round-trip unchanged.
    fn abi_string_unicode_round_trip() {
        let s = "Hőmérséklet: 世界 🦀 аbc"; // Hungarian + CJK + emoji + Cyrillic homoglyph
        let enc = abi_encode_string(s);
        assert_eq!(abi_decode_string(&enc).unwrap(), s);
    }

    #[test]
    // A 64 KiB string (any valid UTF-8) encodes and decodes without panic.
    // On-chain, gas limits constrain string length; off-chain there is no cap.
    fn abi_string_large_round_trip() {
        let s = "x".repeat(65_536);
        let enc = abi_encode_string(&s);
        assert_eq!(abi_decode_string(&enc).unwrap(), s);
    }

    #[test]
    fn abi_empty_string_round_trip() {
        let enc = abi_encode_string("");
        assert_eq!(abi_decode_string(&enc).unwrap(), "");
    }

    // --- ABI tuple ---

    #[test]
    fn abi_tuple_round_trip() {
        let t = AbiTuple {
            a: uint!(999_U256),
            b: address!("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266"),
            c: true,
        };
        let enc = abi_encode_tuple(t);
        let dec = abi_decode_tuple(&enc).unwrap();
        assert_eq!(dec.a, uint!(999_U256));
        assert_eq!(dec.b, address!("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266"));
        assert!(dec.c);
    }

    #[test]
    // U256::MAX must survive an ABI tuple round-trip without truncation.
    fn abi_tuple_max_uint_round_trip() {
        let t = AbiTuple {
            a: U256::MAX,
            b: address!("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266"),
            c: false,
        };
        let enc = abi_encode_tuple(t);
        let dec = abi_decode_tuple(&enc).unwrap();
        assert_eq!(dec.a, U256::MAX);
    }

    // --- RLP u64 ---

    #[test]
    fn rlp_u64_zero() {
        // RLP of integer 0 is a single byte 0x80 (empty string encoding)
        let enc = rlp_encode_u64(0);
        assert_eq!(enc, vec![0x80]);
        assert_eq!(rlp_decode_u64(&enc).unwrap(), 0u64);
    }

    #[test]
    fn rlp_u64_one() {
        // RLP of integer 1 is the single byte 0x01
        let enc = rlp_encode_u64(1);
        assert_eq!(enc, vec![0x01]);
        assert_eq!(rlp_decode_u64(&enc).unwrap(), 1u64);
    }

    #[test]
    // `0xdead_beef` is a classic hex "garbage" value used in low-level debugging
    // (3,735,928,559 decimal). It has no special meaning here — it's just a
    // memorable non-trivial number that exercises more than 1 byte of encoding.
    fn rlp_u64_round_trip() {
        let v = 0xdead_beef_u64;
        let enc = rlp_encode_u64(v);
        assert_eq!(rlp_decode_u64(&enc).unwrap(), v);
    }

    #[test]
    // u64::MAX is the upper boundary — confirms the encoder handles the full 8-byte range.
    fn rlp_u64_max_round_trip() {
        let enc = rlp_encode_u64(u64::MAX);
        assert_eq!(rlp_decode_u64(&enc).unwrap(), u64::MAX);
    }

    #[test]
    // A corrupt / truncated byte slice must return RlpDecodeError, never panic.
    fn rlp_u64_invalid_input_returns_error() {
        let corrupt = vec![0xFF, 0x00]; // Not valid RLP for a u64
        let err = rlp_decode_u64(&corrupt).unwrap_err();
        assert!(matches!(err, PrimitiveError::RlpDecodeError(_)));
    }

    // --- RLP bytes ---

    #[test]
    fn rlp_empty_bytes() {
        // RLP of empty byte array is 0x80
        let enc = rlp_encode_bytes(&[]);
        assert_eq!(enc, vec![0x80]);
        assert_eq!(rlp_decode_bytes(&enc).unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn rlp_bytes_round_trip() {
        let data = vec![0x01, 0x02, 0x03];
        let enc = rlp_encode_bytes(&data);
        assert_eq!(rlp_decode_bytes(&enc).unwrap(), data);
    }

    // --- RLP nested list ---

    #[test]
    // One level of nesting (struct with a u64 and a Vec<u8>) is the representative
    // case for transaction field encoding. Deeper nesting (struct-of-structs) is
    // handled automatically by alloy-rlp's derive macros and is tested as part of
    // full transaction encoding in Phase 2 (T-009 block/receipt indexer).
    fn rlp_list_round_trip() {
        let l = RlpList { a: 42, b: vec![0xca, 0xfe] };
        let enc = rlp_encode_list(&l);
        assert_eq!(rlp_decode_list(&enc).unwrap(), l);
    }
}

/// G7 variant 2: `parse_address` must never panic for any string input.
#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn parse_address_never_panics(s in ".*") {
            // Must always return Ok or Err(PrimitiveError) — never panic.
            let _ = parse_address(&s);
        }
    }
}
