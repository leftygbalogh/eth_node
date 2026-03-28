// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract TestERC721 {
    event Transfer(address indexed from, address indexed to, uint256 indexed tokenId);
    event Approval(address indexed owner, address indexed approved, uint256 indexed tokenId);
    event ApprovalForAll(address indexed owner, address indexed operator, bool approved);

    function emitTransfer(address from, address to, uint256 tokenId) external {
        emit Transfer(from, to, tokenId);
    }

    function emitApproval(address owner, address approved, uint256 tokenId) external {
        emit Approval(owner, approved, tokenId);
    }

    function emitApprovalForAll(address owner, address operator, bool approved) external {
        emit ApprovalForAll(owner, operator, approved);
    }
}
