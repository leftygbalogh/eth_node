//! Phase 2 quality module.
//!
//! T-004: live decode completeness helpers for ERC-721/ERC-1155 event logs.

pub mod decode;

pub use decode::{
    decode_erc1155_approval_for_all, decode_erc721_approval_for_all, decode_standard_nft_event,
    DecodeError, DecodedEvent, Erc1155ApprovalForAllEvent,
    Erc1155TransferBatchEvent, Erc1155TransferSingleEvent, Erc1155UriEvent,
    Erc721ApprovalEvent, Erc721ApprovalForAllEvent, Erc721TransferEvent,
};
