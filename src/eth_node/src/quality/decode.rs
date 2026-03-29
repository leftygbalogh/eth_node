//! Event decode helpers for standard NFT logs.
//!
//! Covers ERC-721 and ERC-1155 standard events for Phase 2 T-004.
//!
//! ## ApprovalForAll Ambiguity
//!
//! The `ApprovalForAll(address,address,bool)` event signature is **shared** between
//! ERC-721 and ERC-1155 standards. When decoding logs from unknown contracts or
//! multi-standard indexers, this creates ambiguity:
//!
//! - [`decode_standard_nft_event`] classifies all ApprovalForAll events as ERC-721
//!   (historical default behavior).
//! - [`decode_nft_event_lossless`] allows explicit standard selection via the
//!   `approval_for_all_as` parameter, or returns `AmbiguousApprovalForAll` when
//!   the standard is unknown.
//!
//! Use the lossless decoder when working with multi-standard contracts or when
//! you need to preserve the ambiguity for downstream resolution.

use alloy_primitives::{Address, B256, U256};
use alloy_rpc_types::Log;
use alloy_sol_types::{sol, SolValue};
use thiserror::Error;

use crate::primitives::event_selector;

sol! {
    struct ApprovalForAllData {
        bool approved;
    }

    struct TransferSingleData {
        uint256 id;
        uint256 value;
    }

    struct TransferBatchData {
        uint256[] ids;
        uint256[] values;
    }

    struct UriData {
        string value;
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum DecodeError {
    #[error("unsupported event topic0: {0:?}")]
    UnsupportedEvent(B256),

    #[error("missing topic at index {0}")]
    MissingTopic(usize),

    #[error("invalid event data: {0}")]
    InvalidData(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Erc721TransferEvent {
    pub from: Address,
    pub to: Address,
    pub token_id: U256,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Erc721ApprovalEvent {
    pub owner: Address,
    pub approved: Address,
    pub token_id: U256,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Erc721ApprovalForAllEvent {
    pub owner: Address,
    pub operator: Address,
    pub approved: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Erc1155TransferSingleEvent {
    pub operator: Address,
    pub from: Address,
    pub to: Address,
    pub id: U256,
    pub value: U256,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Erc1155TransferBatchEvent {
    pub operator: Address,
    pub from: Address,
    pub to: Address,
    pub ids: Vec<U256>,
    pub values: Vec<U256>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Erc1155ApprovalForAllEvent {
    pub account: Address,
    pub operator: Address,
    pub approved: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Erc1155UriEvent {
    pub value: String,
    pub id: U256,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DecodedEvent {
    Erc721Transfer(Erc721TransferEvent),
    Erc721Approval(Erc721ApprovalEvent),
    Erc721ApprovalForAll(Erc721ApprovalForAllEvent),
    Erc1155TransferSingle(Erc1155TransferSingleEvent),
    Erc1155TransferBatch(Erc1155TransferBatchEvent),
    Erc1155ApprovalForAll(Erc1155ApprovalForAllEvent),
    Erc1155Uri(Erc1155UriEvent),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalForAllStandard {
    Erc721,
    Erc1155,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AmbiguousApprovalForAllEvent {
    pub subject: Address,
    pub operator: Address,
    pub approved: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LosslessDecodedEvent {
    Decoded(DecodedEvent),
    AmbiguousApprovalForAll(AmbiguousApprovalForAllEvent),
}

/// Decode standard NFT events (ERC-721 and ERC-1155) from a log entry.
///
/// Supports: Transfer, Approval (ERC-721), TransferSingle, TransferBatch,
/// ApprovalForAll (classified as ERC-721 by default), and URI (ERC-1155).
///
/// # Arguments
///
/// - `log`: Reference to an [`alloy_rpc_types::Log`] containing the event data.
///
/// # Returns
///
/// - `Ok(DecodedEvent)` if the event signature matches a known standard and decodes successfully.
/// - `Err(DecodeError::UnsupportedEvent)` if topic0 is not a recognized NFT event.
/// - `Err(DecodeError::MissingTopic)` if required topics are missing.
/// - `Err(DecodeError::InvalidData)` if event data cannot be ABI-decoded.
///
/// # Example
///
/// ```ignore
/// use eth_node::quality::decode_standard_nft_event;
/// use alloy_rpc_types::Log;
///
/// let decoded = decode_standard_nft_event(&log)?;
/// match decoded {
///     DecodedEvent::Erc721Transfer(evt) => println!("Token {} transferred", evt.token_id),
///     _ => println!("Other event"),
/// }
/// ```
pub fn decode_standard_nft_event(log: &Log) -> Result<DecodedEvent, DecodeError> {
    let topic0 = *log.topic0().ok_or(DecodeError::MissingTopic(0))?;

    if topic0 == event_selector("Transfer(address,address,uint256)") {
        return decode_erc721_transfer(log).map(DecodedEvent::Erc721Transfer);
    }
    if topic0 == event_selector("Approval(address,address,uint256)") {
        return decode_erc721_approval(log).map(DecodedEvent::Erc721Approval);
    }
    if topic0 == event_selector("ApprovalForAll(address,address,bool)") {
        // Shared signature between ERC-721 and ERC-1155; classify based on topic/data shape.
        return decode_shared_approval_for_all(log);
    }
    if topic0 == event_selector("TransferSingle(address,address,address,uint256,uint256)") {
        return decode_erc1155_transfer_single(log).map(DecodedEvent::Erc1155TransferSingle);
    }
    if topic0 == event_selector("TransferBatch(address,address,address,uint256[],uint256[])") {
        return decode_erc1155_transfer_batch(log).map(DecodedEvent::Erc1155TransferBatch);
    }
    if topic0 == event_selector("URI(string,uint256)") {
        return decode_erc1155_uri(log).map(DecodedEvent::Erc1155Uri);
    }

    Err(DecodeError::UnsupportedEvent(topic0))
}

/// Decode NFT events without information loss for ApprovalForAll ambiguity.
///
/// Identical to [`decode_standard_nft_event`] except for `ApprovalForAll(address,address,bool)`:
/// - If `approval_for_all_as` is `Some(standard)`, decodes as the specified standard (ERC-721 or ERC-1155).
/// - If `None`, returns `LosslessDecodedEvent::AmbiguousApprovalForAll` preserving the raw data
///   for downstream resolution (e.g., contract introspection or heuristic classification).
///
/// # Arguments
///
/// - `log`: Reference to an [`alloy_rpc_types::Log`] containing the event data.
/// - `approval_for_all_as`: Optional hint for ApprovalForAll standard classification.
///   - `Some(ApprovalForAllStandard::Erc721)` → decode as ERC-721 event.
///   - `Some(ApprovalForAllStandard::Erc1155)` → decode as ERC-1155 event.
///   - `None` → return ambiguous event for external resolution.
///
/// # Returns
///
/// - `Ok(LosslessDecodedEvent::Decoded(event))` for unambiguous events.
/// - `Ok(LosslessDecodedEvent::AmbiguousApprovalForAll(event))` when ApprovalForAll standard is unknown.
/// - `Err(DecodeError)` if decoding fails (same error conditions as `decode_standard_nft_event`).
///
/// # Example
///
/// ```ignore
/// use eth_node::quality::{decode_nft_event_lossless, ApprovalForAllStandard};
/// use alloy_rpc_types::Log;
///
/// // Explicitly classify as ERC-1155:
/// let decoded = decode_nft_event_lossless(&log, Some(ApprovalForAllStandard::Erc1155))?;
///
/// // Preserve ambiguity:
/// let decoded = decode_nft_event_lossless(&log, None)?;
/// match decoded {
///     LosslessDecodedEvent::AmbiguousApprovalForAll(evt) => {
///         println!("Ambiguous ApprovalForAll from {}", evt.subject);
///     }
///     _ => {},
/// }
/// ```
pub fn decode_nft_event_lossless(
    log: &Log,
    approval_for_all_as: Option<ApprovalForAllStandard>,
) -> Result<LosslessDecodedEvent, DecodeError> {
    let topic0 = *log.topic0().ok_or(DecodeError::MissingTopic(0))?;

    if topic0 == event_selector("ApprovalForAll(address,address,bool)") {
        return match approval_for_all_as {
            Some(ApprovalForAllStandard::Erc721) => decode_erc721_approval_for_all(log)
                .map(DecodedEvent::Erc721ApprovalForAll)
                .map(LosslessDecodedEvent::Decoded),
            Some(ApprovalForAllStandard::Erc1155) => decode_erc1155_approval_for_all(log)
                .map(DecodedEvent::Erc1155ApprovalForAll)
                .map(LosslessDecodedEvent::Decoded),
            None => decode_erc721_approval_for_all(log)
                .map(|event| {
                    LosslessDecodedEvent::AmbiguousApprovalForAll(AmbiguousApprovalForAllEvent {
                        subject: event.owner,
                        operator: event.operator,
                        approved: event.approved,
                    })
                }),
        };
    }

    decode_standard_nft_event(log).map(LosslessDecodedEvent::Decoded)
}

fn decode_erc721_transfer(log: &Log) -> Result<Erc721TransferEvent, DecodeError> {
    Ok(Erc721TransferEvent {
        from: topic_to_address(log.topics().get(1), 1)?,
        to: topic_to_address(log.topics().get(2), 2)?,
        token_id: topic_to_u256(log.topics().get(3), 3)?,
    })
}

fn decode_erc721_approval(log: &Log) -> Result<Erc721ApprovalEvent, DecodeError> {
    Ok(Erc721ApprovalEvent {
        owner: topic_to_address(log.topics().get(1), 1)?,
        approved: topic_to_address(log.topics().get(2), 2)?,
        token_id: topic_to_u256(log.topics().get(3), 3)?,
    })
}

fn decode_shared_approval_for_all(log: &Log) -> Result<DecodedEvent, DecodeError> {
    decode_erc721_approval_for_all(log).map(DecodedEvent::Erc721ApprovalForAll)
}

/// Decode an `ApprovalForAll(address,address,bool)` event as ERC-721.
///
/// Extracts `owner` and `operator` from topics 1 and 2, and `approved` from event data.
///
/// # Arguments
///
/// - `log`: Reference to an [`alloy_rpc_types::Log`] with ApprovalForAll event data.
///
/// # Returns
///
/// - `Ok(Erc721ApprovalForAllEvent)` if topics and data decode successfully.
/// - `Err(DecodeError::MissingTopic)` if required topics are missing.
/// - `Err(DecodeError::InvalidData)` if the boolean data field cannot be ABI-decoded.
///
/// # Example
///
/// ```ignore
/// use eth_node::quality::decode_erc721_approval_for_all;
/// use alloy_rpc_types::Log;
///
/// let event = decode_erc721_approval_for_all(&log)?;
/// println!("Owner {} approved operator {}: {}", event.owner, event.operator, event.approved);
/// ```
pub fn decode_erc721_approval_for_all(
    log: &Log,
) -> Result<Erc721ApprovalForAllEvent, DecodeError> {
    let data = ApprovalForAllData::abi_decode(log.data().data.as_ref(), true)
        .map_err(|e| DecodeError::InvalidData(format!("approvalForAll decode failed: {e}")))?;

    Ok(Erc721ApprovalForAllEvent {
        owner: topic_to_address(log.topics().get(1), 1)?,
        operator: topic_to_address(log.topics().get(2), 2)?,
        approved: data.approved,
    })
}

/// Decode an `ApprovalForAll(address,address,bool)` event as ERC-1155.
///
/// Extracts `account` and `operator` from topics 1 and 2, and `approved` from event data.
/// Semantically identical to [`decode_erc721_approval_for_all`] but returns ERC-1155 event type.
///
/// # Arguments
///
/// - `log`: Reference to an [`alloy_rpc_types::Log`] with ApprovalForAll event data.
///
/// # Returns
///
/// - `Ok(Erc1155ApprovalForAllEvent)` if topics and data decode successfully.
/// - `Err(DecodeError::MissingTopic)` if required topics are missing.
/// - `Err(DecodeError::InvalidData)` if the boolean data field cannot be ABI-decoded.
///
/// # Example
///
/// ```ignore
/// use eth_node::quality::decode_erc1155_approval_for_all;
/// use alloy_rpc_types::Log;
///
/// let event = decode_erc1155_approval_for_all(&log)?;
/// println!("Account {} approved operator {}: {}", event.account, event.operator, event.approved);
/// ```
pub fn decode_erc1155_approval_for_all(
    log: &Log,
) -> Result<Erc1155ApprovalForAllEvent, DecodeError> {
    let data = ApprovalForAllData::abi_decode(log.data().data.as_ref(), true)
        .map_err(|e| DecodeError::InvalidData(format!("approvalForAll decode failed: {e}")))?;

    Ok(Erc1155ApprovalForAllEvent {
        account: topic_to_address(log.topics().get(1), 1)?,
        operator: topic_to_address(log.topics().get(2), 2)?,
        approved: data.approved,
    })
}

fn decode_erc1155_transfer_single(log: &Log) -> Result<Erc1155TransferSingleEvent, DecodeError> {
    let data = TransferSingleData::abi_decode(log.data().data.as_ref(), true)
        .map_err(|e| DecodeError::InvalidData(format!("transferSingle decode failed: {e}")))?;

    Ok(Erc1155TransferSingleEvent {
        operator: topic_to_address(log.topics().get(1), 1)?,
        from: topic_to_address(log.topics().get(2), 2)?,
        to: topic_to_address(log.topics().get(3), 3)?,
        id: data.id,
        value: data.value,
    })
}

fn decode_erc1155_transfer_batch(log: &Log) -> Result<Erc1155TransferBatchEvent, DecodeError> {
    let (ids, values) = match TransferBatchData::abi_decode(log.data().data.as_ref(), true) {
        Ok(data) => (
            data.ids.into_iter().collect::<Vec<U256>>(),
            data.values.into_iter().collect::<Vec<U256>>(),
        ),
        Err(primary_err) => {
            // Live-chain logs may require direct tuple word parsing for dynamic arrays.
            parse_u256_array_pair(log.data().data.as_ref()).map_err(|secondary_err| {
                DecodeError::InvalidData(format!(
                    "transferBatch decode failed: {primary_err}; fallback failed: {secondary_err}"
                ))
            })?
        }
    };

    Ok(Erc1155TransferBatchEvent {
        operator: topic_to_address(log.topics().get(1), 1)?,
        from: topic_to_address(log.topics().get(2), 2)?,
        to: topic_to_address(log.topics().get(3), 3)?,
        ids,
        values,
    })
}

fn decode_erc1155_uri(log: &Log) -> Result<Erc1155UriEvent, DecodeError> {
    let value = match UriData::abi_decode(log.data().data.as_ref(), true) {
        Ok(data) => data.value,
        Err(primary_err) => {
            // Live-chain logs may present URI payload as direct ABI string bytes.
            String::abi_decode(log.data().data.as_ref(), true).map_err(|secondary_err| {
                DecodeError::InvalidData(format!(
                    "uri decode failed: {primary_err}; fallback failed: {secondary_err}"
                ))
            })?
        }
    };

    Ok(Erc1155UriEvent {
        value,
        id: topic_to_u256(log.topics().get(1), 1)?,
    })
}

fn topic_to_address(topic: Option<&B256>, index: usize) -> Result<Address, DecodeError> {
    let topic = topic.ok_or(DecodeError::MissingTopic(index))?;
    let bytes = topic.as_slice();
    Ok(Address::from_slice(&bytes[12..]))
}

fn topic_to_u256(topic: Option<&B256>, index: usize) -> Result<U256, DecodeError> {
    let topic = topic.ok_or(DecodeError::MissingTopic(index))?;
    Ok(U256::from_be_slice(topic.as_slice()))
}

fn parse_u256_array_pair(data: &[u8]) -> Result<(Vec<U256>, Vec<U256>), String> {
    let first_offset = read_word_as_usize(data, 0)?;
    let second_offset = read_word_as_usize(data, 32)?;

    let ids = read_u256_array_at(data, first_offset)?;
    let values = read_u256_array_at(data, second_offset)?;

    Ok((ids, values))
}

fn read_word_as_usize(data: &[u8], offset: usize) -> Result<usize, String> {
    if data.len() < offset + 32 {
        return Err(format!("buffer too short reading word at offset {offset}"));
    }
    let word = &data[offset..offset + 32];
    let mut out: usize = 0;
    for byte in word {
        out = out
            .checked_mul(256)
            .ok_or_else(|| format!("offset overflow while parsing word at {offset}"))?;
        out = out
            .checked_add(*byte as usize)
            .ok_or_else(|| format!("offset overflow while parsing word at {offset}"))?;
    }
    Ok(out)
}

fn read_u256_array_at(data: &[u8], offset: usize) -> Result<Vec<U256>, String> {
    let len = read_word_as_usize(data, offset)?;
    let mut out = Vec::with_capacity(len);
    let mut cursor = offset
        .checked_add(32)
        .ok_or_else(|| "array cursor overflow".to_string())?;

    for _ in 0..len {
        if data.len() < cursor + 32 {
            return Err("buffer too short reading dynamic array value".to_string());
        }
        out.push(U256::from_be_slice(&data[cursor..cursor + 32]));
        cursor = cursor
            .checked_add(32)
            .ok_or_else(|| "array cursor overflow".to_string())?;
    }
    Ok(out)
}
