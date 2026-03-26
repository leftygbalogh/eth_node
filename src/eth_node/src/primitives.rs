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
    //Lefty: we have a few wrong format cases below, 
    // it would be nice to see what happens with all upper case and with mixed case addresses.
    fn address_from_hex_lowercase() {
        // lowercase is valid — just verify it parses without error
        assert!(parse_address("0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266").is_ok());
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
    //Lefty: for my sake, can we add U256::MAX with U256::MAX 
    // twice using + and using checked_add
    // and check its results?
    fn u256_checked_add_overflow() {
        assert!(U256::MAX.checked_add(uint!(1_U256)).is_none());
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
    //Lefty: is this the only known vector? 
    // Do we have a more complete list of malicious vectors we should be able to handle?
    fn abi_uint256_known_vector() {
        // uint256(1) → 31 zero bytes then 0x01
        let enc = abi_encode_uint256(uint!(1_U256));
        assert_eq!(enc.len(), 32);
        assert_eq!(&enc[..31], &[0u8; 31]);
        assert_eq!(enc[31], 1u8);
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
    //Lefty: what is the longest string we should be able to handle?
    //What happens with strings that are longer? For exmpe someone pasting in the entire Bible.
    //what happens with smilies, emoticons, hungarian letter with weird diacritical marks, nonsimplified chinese characters, homoglyphs?
    fn abi_string_round_trip() {
        let s = "hello, ethereum";
        let enc = abi_encode_string(s);
        assert_eq!(abi_decode_string(&enc).unwrap(), s);
    }

    #[test]
    fn abi_empty_string_round_trip() {
        let enc = abi_encode_string("");
        assert_eq!(abi_decode_string(&enc).unwrap(), "");
    }

    // --- ABI tuple ---

    #[test]
    //Lefty: for this  one, it would be interesting to see if the uint.MAX value makes the roundtrip
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
    //Lefty: what is this one (dead_beef - is this a hexadecimal number here?) for? 
    //should we try more variants that push harder around edges and limit or type violations?
    // I would like to see how we handle invalid inputs
    fn rlp_u64_round_trip() {
        let v = 0xdead_beef_u64;
        let enc = rlp_encode_u64(v);
        assert_eq!(rlp_decode_u64(&enc).unwrap(), v);
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
    //Lefty: did we want to try more compex data structures or are they not relevant or allowed?
    fn rlp_list_round_trip() {
        let l = RlpList { a: 42, b: vec![0xca, 0xfe] };
        let enc = rlp_encode_list(&l);
        assert_eq!(rlp_decode_list(&enc).unwrap(), l);
    }
}
