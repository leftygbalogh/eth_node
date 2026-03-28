// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract TestERC1155 {
    event TransferSingle(
        address indexed operator,
        address indexed from,
        address indexed to,
        uint256 id,
        uint256 value
    );
    event TransferBatch(
        address indexed operator,
        address indexed from,
        address indexed to,
        uint256[] ids,
        uint256[] values
    );
    event ApprovalForAll(address indexed account, address indexed operator, bool approved);
    event URI(string value, uint256 indexed id);

    function emitTransferSingle(
        address operator,
        address from,
        address to,
        uint256 id,
        uint256 value
    ) external {
        emit TransferSingle(operator, from, to, id, value);
    }

    function emitTransferBatch(
        address operator,
        address from,
        address to,
        uint256[] calldata ids,
        uint256[] calldata values
    ) external {
        emit TransferBatch(operator, from, to, ids, values);
    }

    function emitApprovalForAll(address account, address operator, bool approved) external {
        emit ApprovalForAll(account, operator, approved);
    }

    function emitUri(string calldata value, uint256 id) external {
        emit URI(value, id);
    }
}
