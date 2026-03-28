//! T-004 decode completeness tests.
//!
//! NOTE: These tests exercise real ABI/topic decoding paths on canonical event
//! payloads and edge-case payloads. Contract deployment-based live tests can be
//! layered on top once Solidity build/deploy fixtures are introduced.

use alloy_primitives::{Address, B256, Bytes, Log as PrimitiveLog, U256};
use alloy_rpc_types::Log;
use alloy_sol_types::{sol, SolValue};
use eth_node::primitives::event_selector;
use eth_node::quality::{
    decode_erc1155_approval_for_all, decode_erc721_approval_for_all,
    decode_standard_nft_event, DecodeError, DecodedEvent,
};

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

fn test_emitter() -> Address {
    Address::repeat_byte(0x11)
}

fn addr_topic(addr: Address) -> B256 {
    let mut out = [0u8; 32];
    out[12..].copy_from_slice(addr.as_slice());
    B256::from(out)
}

fn u256_topic(v: U256) -> B256 {
    B256::from(v.to_be_bytes::<32>())
}

fn mk_log(topics: Vec<B256>, data: Bytes) -> Log {
    Log {
        inner: PrimitiveLog::new_unchecked(test_emitter(), topics, data),
        ..Default::default()
    }
}

// ----- ERC-721 (5 tests) -----

#[test]
fn decode_erc721_transfer_standard() {
    let from = Address::repeat_byte(0x21);
    let to = Address::repeat_byte(0x22);
    let token_id = U256::from(42);

    let log = mk_log(
        vec![
            event_selector("Transfer(address,address,uint256)"),
            addr_topic(from),
            addr_topic(to),
            u256_topic(token_id),
        ],
        Bytes::new(),
    );

    let decoded = decode_standard_nft_event(&log).expect("decode transfer");
    assert_eq!(
        decoded,
        DecodedEvent::Erc721Transfer(eth_node::quality::Erc721TransferEvent { from, to, token_id })
    );
}

#[test]
fn decode_erc721_approval_standard() {
    let owner = Address::repeat_byte(0x31);
    let approved = Address::repeat_byte(0x32);
    let token_id = U256::from(99);

    let log = mk_log(
        vec![
            event_selector("Approval(address,address,uint256)"),
            addr_topic(owner),
            addr_topic(approved),
            u256_topic(token_id),
        ],
        Bytes::new(),
    );

    let decoded = decode_standard_nft_event(&log).expect("decode approval");
    assert_eq!(
        decoded,
        DecodedEvent::Erc721Approval(eth_node::quality::Erc721ApprovalEvent {
            owner,
            approved,
            token_id,
        })
    );
}

#[test]
fn decode_erc721_approval_for_all_true() {
    let owner = Address::repeat_byte(0x41);
    let operator = Address::repeat_byte(0x42);

    let data = ApprovalForAllData { approved: true }.abi_encode();
    let log = mk_log(
        vec![
            event_selector("ApprovalForAll(address,address,bool)"),
            addr_topic(owner),
            addr_topic(operator),
        ],
        Bytes::from(data),
    );

    let decoded = decode_erc721_approval_for_all(&log).expect("decode approvalForAll");
    assert!(decoded.approved);
    assert_eq!(decoded.owner, owner);
    assert_eq!(decoded.operator, operator);
}

#[test]
fn decode_erc721_transfer_self_transfer_edge_case() {
    // Edge case AC-010: self-transfer where from == to.
    let user = Address::repeat_byte(0x55);
    let token_id = U256::from(1);

    let log = mk_log(
        vec![
            event_selector("Transfer(address,address,uint256)"),
            addr_topic(user),
            addr_topic(user),
            u256_topic(token_id),
        ],
        Bytes::new(),
    );

    let decoded = decode_standard_nft_event(&log).expect("decode self transfer");
    match decoded {
        DecodedEvent::Erc721Transfer(e) => {
            assert_eq!(e.from, user);
            assert_eq!(e.to, user);
            assert_eq!(e.token_id, token_id);
        }
        other => panic!("unexpected event variant: {other:?}"),
    }
}

#[test]
fn decode_erc721_transfer_max_token_id_edge_case() {
    // Edge case AC-010: maximum uint256 token id.
    let from = Address::repeat_byte(0x61);
    let to = Address::repeat_byte(0x62);
    let token_id = U256::MAX;

    let log = mk_log(
        vec![
            event_selector("Transfer(address,address,uint256)"),
            addr_topic(from),
            addr_topic(to),
            u256_topic(token_id),
        ],
        Bytes::new(),
    );

    let decoded = decode_standard_nft_event(&log).expect("decode max token id");
    match decoded {
        DecodedEvent::Erc721Transfer(e) => assert_eq!(e.token_id, U256::MAX),
        other => panic!("unexpected event variant: {other:?}"),
    }
}

// ----- ERC-1155 (10 tests) -----

#[test]
fn decode_erc1155_transfer_single_standard() {
    let operator = Address::repeat_byte(0x71);
    let from = Address::repeat_byte(0x72);
    let to = Address::repeat_byte(0x73);

    let data = TransferSingleData {
        id: U256::from(7),
        value: U256::from(300),
    }
    .abi_encode();

    let log = mk_log(
        vec![
            event_selector("TransferSingle(address,address,address,uint256,uint256)"),
            addr_topic(operator),
            addr_topic(from),
            addr_topic(to),
        ],
        Bytes::from(data),
    );

    let decoded = decode_standard_nft_event(&log).expect("decode transferSingle");
    match decoded {
        DecodedEvent::Erc1155TransferSingle(e) => {
            assert_eq!(e.operator, operator);
            assert_eq!(e.from, from);
            assert_eq!(e.to, to);
            assert_eq!(e.id, U256::from(7));
            assert_eq!(e.value, U256::from(300));
        }
        other => panic!("unexpected event variant: {other:?}"),
    }
}

#[test]
fn decode_erc1155_transfer_batch_standard() {
    let operator = Address::repeat_byte(0x81);
    let from = Address::repeat_byte(0x82);
    let to = Address::repeat_byte(0x83);

    let data = TransferBatchData {
        ids: vec![U256::from(1), U256::from(2), U256::from(3)],
        values: vec![U256::from(10), U256::from(20), U256::from(30)],
    }
    .abi_encode();

    let log = mk_log(
        vec![
            event_selector("TransferBatch(address,address,address,uint256[],uint256[])"),
            addr_topic(operator),
            addr_topic(from),
            addr_topic(to),
        ],
        Bytes::from(data),
    );

    let decoded = decode_standard_nft_event(&log).expect("decode transferBatch");
    match decoded {
        DecodedEvent::Erc1155TransferBatch(e) => {
            assert_eq!(e.ids.len(), 3);
            assert_eq!(e.values.len(), 3);
            assert_eq!(e.ids[0], U256::from(1));
            assert_eq!(e.values[2], U256::from(30));
        }
        other => panic!("unexpected event variant: {other:?}"),
    }
}

#[test]
fn decode_erc1155_transfer_batch_empty_arrays_edge_case() {
    // Edge case AC-010: TransferBatch with empty ids/values arrays.
    let operator = Address::repeat_byte(0x84);
    let from = Address::repeat_byte(0x85);
    let to = Address::repeat_byte(0x86);

    let data = TransferBatchData {
        ids: vec![],
        values: vec![],
    }
    .abi_encode();

    let log = mk_log(
        vec![
            event_selector("TransferBatch(address,address,address,uint256[],uint256[])"),
            addr_topic(operator),
            addr_topic(from),
            addr_topic(to),
        ],
        Bytes::from(data),
    );

    let decoded = decode_standard_nft_event(&log).expect("decode empty transferBatch");
    match decoded {
        DecodedEvent::Erc1155TransferBatch(e) => {
            assert!(e.ids.is_empty());
            assert!(e.values.is_empty());
        }
        other => panic!("unexpected event variant: {other:?}"),
    }
}

#[test]
fn decode_erc1155_transfer_single_zero_value_edge_case() {
    // Edge case AC-010: zero-value transfer.
    let operator = Address::repeat_byte(0x87);
    let from = Address::repeat_byte(0x88);
    let to = Address::repeat_byte(0x89);

    let data = TransferSingleData {
        id: U256::from(55),
        value: U256::ZERO,
    }
    .abi_encode();

    let log = mk_log(
        vec![
            event_selector("TransferSingle(address,address,address,uint256,uint256)"),
            addr_topic(operator),
            addr_topic(from),
            addr_topic(to),
        ],
        Bytes::from(data),
    );

    let decoded = decode_standard_nft_event(&log).expect("decode zero-value transferSingle");
    match decoded {
        DecodedEvent::Erc1155TransferSingle(e) => assert_eq!(e.value, U256::ZERO),
        other => panic!("unexpected event variant: {other:?}"),
    }
}

#[test]
fn decode_erc1155_transfer_single_self_transfer_edge_case() {
    // Edge case AC-010: self-transfer where from == to.
    let operator = Address::repeat_byte(0x8a);
    let holder = Address::repeat_byte(0x8b);

    let data = TransferSingleData {
        id: U256::from(999),
        value: U256::from(1),
    }
    .abi_encode();

    let log = mk_log(
        vec![
            event_selector("TransferSingle(address,address,address,uint256,uint256)"),
            addr_topic(operator),
            addr_topic(holder),
            addr_topic(holder),
        ],
        Bytes::from(data),
    );

    let decoded = decode_standard_nft_event(&log).expect("decode self transferSingle");
    match decoded {
        DecodedEvent::Erc1155TransferSingle(e) => {
            assert_eq!(e.from, holder);
            assert_eq!(e.to, holder);
        }
        other => panic!("unexpected event variant: {other:?}"),
    }
}

#[test]
fn decode_erc1155_transfer_single_max_id_edge_case() {
    // Edge case AC-010: maximum uint256 token id.
    let operator = Address::repeat_byte(0x8c);
    let from = Address::repeat_byte(0x8d);
    let to = Address::repeat_byte(0x8e);

    let data = TransferSingleData {
        id: U256::MAX,
        value: U256::from(2),
    }
    .abi_encode();

    let log = mk_log(
        vec![
            event_selector("TransferSingle(address,address,address,uint256,uint256)"),
            addr_topic(operator),
            addr_topic(from),
            addr_topic(to),
        ],
        Bytes::from(data),
    );

    let decoded = decode_standard_nft_event(&log).expect("decode max id transferSingle");
    match decoded {
        DecodedEvent::Erc1155TransferSingle(e) => assert_eq!(e.id, U256::MAX),
        other => panic!("unexpected event variant: {other:?}"),
    }
}

#[test]
fn decode_erc1155_uri_standard() {
    let id = U256::from(501);
    let uri = "ipfs://token/501".to_string();
    let data = UriData { value: uri.clone() }.abi_encode();

    let log = mk_log(
        vec![event_selector("URI(string,uint256)"), u256_topic(id)],
        Bytes::from(data),
    );

    let decoded = decode_standard_nft_event(&log).expect("decode URI event");
    match decoded {
        DecodedEvent::Erc1155Uri(e) => {
            assert_eq!(e.id, id);
            assert_eq!(e.value, uri);
        }
        other => panic!("unexpected event variant: {other:?}"),
    }
}

#[test]
fn decode_erc1155_uri_max_id_edge_case() {
    let data = UriData {
        value: "ipfs://token/max".to_string(),
    }
    .abi_encode();

    let log = mk_log(
        vec![event_selector("URI(string,uint256)"), u256_topic(U256::MAX)],
        Bytes::from(data),
    );

    let decoded = decode_standard_nft_event(&log).expect("decode max-id URI event");
    match decoded {
        DecodedEvent::Erc1155Uri(e) => assert_eq!(e.id, U256::MAX),
        other => panic!("unexpected event variant: {other:?}"),
    }
}

#[test]
fn decode_erc1155_approval_for_all_false() {
    let account = Address::repeat_byte(0x91);
    let operator = Address::repeat_byte(0x92);
    let data = ApprovalForAllData { approved: false }.abi_encode();

    let log = mk_log(
        vec![
            event_selector("ApprovalForAll(address,address,bool)"),
            addr_topic(account),
            addr_topic(operator),
        ],
        Bytes::from(data),
    );

    let decoded = decode_erc1155_approval_for_all(&log).expect("decode erc1155 approvalForAll");
    assert_eq!(decoded.account, account);
    assert_eq!(decoded.operator, operator);
    assert!(!decoded.approved);
}

#[test]
fn decode_returns_unsupported_for_unknown_signature() {
    let log = mk_log(vec![B256::repeat_byte(0xaa)], Bytes::new());

    let err = decode_standard_nft_event(&log).expect_err("expected unsupported event error");
    assert!(matches!(err, DecodeError::UnsupportedEvent(_)));
}

#[test]
fn decode_returns_missing_topic_error_when_required_topic_absent() {
    let log = mk_log(vec![event_selector("Transfer(address,address,uint256)")], Bytes::new());

    let err = decode_standard_nft_event(&log).expect_err("expected missing topic error");
    assert!(matches!(err, DecodeError::MissingTopic(1)));
}
