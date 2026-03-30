//! Example: NFT Event Decoding
//!
//! This example demonstrates decoding ERC-721 and ERC-1155 events from logs,
//! including handling of ApprovalForAll ambiguity.
//!
//! Run with:
//! ```bash
//! cargo run --example decode_nft_events
//! ```

use eth_node::quality::{
    decode_standard_nft_event, decode_nft_event_lossless,
    DecodedEvent, LosslessDecodedEvent, ApprovalForAllStandard,
};
use alloy_rpc_types::Log;
use alloy_primitives::{Address, U256, Bytes, B256, FixedBytes, Log as PrimitiveLog};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== NFT Event Decoding Example ===\n");

    // ERC-721 Transfer event signature
    let erc721_transfer_topic = FixedBytes::<32>::from([
        0xdd, 0xf2, 0x52, 0xad, 0x1b, 0xe2, 0xc8, 0x9b,
        0x69, 0xc2, 0xb0, 0x68, 0xfc, 0x37, 0x8d, 0xaa,
        0x95, 0x2b, 0xa7, 0xf1, 0x63, 0xc4, 0xa1, 0x16,
        0x28, 0xf5, 0x5a, 0x4d, 0xf5, 0x23, 0xb3, 0xef,
    ]);

    // Example 1: Decode ERC-721 Transfer event
    println!("Example 1: ERC-721 Transfer");
    println!("-----------------------------");

    let erc721_log = Log {
        inner: PrimitiveLog::new_unchecked(
            Address::from([0xAA; 20]),
            vec![
                erc721_transfer_topic,
                B256::left_padding_from(&[0x11; 20]), // from
                B256::left_padding_from(&[0x22; 20]), // to
            ],
            Bytes::from(U256::from(42).to_be_bytes_vec()), // tokenId
        ),
        ..Default::default()
    };

    match decode_standard_nft_event(&erc721_log)? {
        DecodedEvent::Erc721Transfer(evt) => {
            println!("✓ Decoded ERC-721 Transfer:");
            println!("  Token ID: {}", evt.token_id);
            println!("  From:     {:?}", evt.from);
            println!("  To:       {:?}", evt.to);
        }
        _ => println!("Unexpected event type"),
    }

    // Example 2: Handle ApprovalForAll ambiguity
    println!("\n\nExample 2: ApprovalForAll Ambiguity");
    println!("-------------------------------------");

    let approval_for_all_topic = FixedBytes::<32>::from([
        0x17, 0x30, 0x7e, 0xab, 0x39, 0xab, 0x61, 0x07,
        0xe8, 0x89, 0x98, 0x45, 0xad, 0x3d, 0x59, 0xbd,
        0x96, 0x53, 0xf2, 0x00, 0xf2, 0x20, 0x92, 0x04,
        0x89, 0xca, 0x2b, 0x59, 0x37, 0x69, 0x6c, 0x31,
    ]);

    let ambiguous_log = Log {
        inner: PrimitiveLog::new_unchecked(
            Address::from([0xBB; 20]),
            vec![
                approval_for_all_topic,
                B256::left_padding_from(&[0x33; 20]), // owner/account
                B256::left_padding_from(&[0x44; 20]), // operator
            ],
            Bytes::from([0x01]), // approved = true
        ),
        ..Default::default()
    };

    println!("This event has identical signature in ERC-721 and ERC-1155:");
    println!("  ERC-721:  ApprovalForAll(address indexed owner, ...)");
    println!("  ERC-1155: ApprovalForAll(address indexed account, ...)");
    println!("\nOnly semantic meaning of topic[1] differs (owner vs account)\n");

    // Option A: Assume ERC-721 (historical default)
    println!("Option A: Decode as ERC-721 (default assumption)");
    match decode_standard_nft_event(&ambiguous_log)? {
        DecodedEvent::Erc721ApprovalForAll(evt) => {
            println!("✓ Decoded as ERC-721 ApprovalForAll:");
            println!("  Owner:    {:?}", evt.owner);
            println!("  Operator: {:?}", evt.operator);
            println!("  Approved: {}", evt.approved);
        }
        _ => println!("Unexpected event type"),
    }

    // Option B: Explicitly decode as ERC-1155
    println!("\nOption B: Explicitly decode as ERC-1155");
    match decode_nft_event_lossless(&ambiguous_log, Some(ApprovalForAllStandard::Erc1155))? {
        LosslessDecodedEvent::Decoded(DecodedEvent::Erc1155ApprovalForAll(evt)) => {
            println!("✓ Decoded as ERC-1155 ApprovalForAll:");
            println!("  Account:  {:?}", evt.account);
            println!("  Operator: {:?}", evt.operator);
            println!("  Approved: {}", evt.approved);
        }
        _ => println!("Unexpected event type"),
    }

    // Option C: Preserve ambiguity for downstream resolution
    println!("\nOption C: Preserve ambiguity (no assumed standard)");
    match decode_nft_event_lossless(&ambiguous_log, None)? {
        LosslessDecodedEvent::AmbiguousApprovalForAll(evt) => {
            println!("✓ Ambiguous ApprovalForAll preserved:");
            println!("  Subject:  {:?}", evt.subject);
            println!("  Operator: {:?}", evt.operator);
            println!("  Approved: {}", evt.approved);
            println!("\n  → Downstream can resolve via ERC-165 supportsInterface()");
        }
        _ => println!("Unexpected event type"),
    }

    println!("\n\nKey Points:");
    println!("  • Use decode_standard_nft_event() when standard is known (assumes ERC-721 for ApprovalForAll)");
    println!("  • Use decode_nft_event_lossless() with explicit standard when you know the contract type");
    println!("  • Use decode_nft_event_lossless(log, None) to preserve ambiguity for downstream resolution");
    println!("  • Query contract via ERC-165 to definitively classify ApprovalForAll events");

    Ok(())
}
