// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/**
 * @title SimpleNFT
 * @notice Minimal ERC-721-like NFT for documentation examples
 * @dev Not production-ready - for learning/testing only
 */
contract SimpleNFT {
    // Token ownership
    mapping(uint256 => address) private _owners;
    
    // Token approvals
    mapping(uint256 => address) private _tokenApprovals;
    
    // Operator approvals
    mapping(address => mapping(address => bool)) private _operatorApprovals;
    
    // ERC-721 events
    event Transfer(address indexed from, address indexed to, uint256 indexed tokenId);
    event Approval(address indexed owner, address indexed approved, uint256 indexed tokenId);
    event ApprovalForAll(address indexed owner, address indexed operator, bool approved);
    
    /**
     * @notice Mint a new token to a specific address
     * @param to Recipient address
     * @param tokenId Token ID to mint
     */
    function mint(address to, uint256 tokenId) external {
        require(to != address(0), "mint to zero address");
        require(_owners[tokenId] == address(0), "token already minted");
        
        _owners[tokenId] = to;
        emit Transfer(address(0), to, tokenId);
    }
    
    /**
     * @notice Transfer token from one address to another
     * @param from Current owner
     * @param to Recipient
     * @param tokenId Token to transfer
     */
    function transferFrom(address from, address to, uint256 tokenId) external {
        require(_owners[tokenId] == from, "from is not owner");
        require(to != address(0), "transfer to zero address");
        
        // Simple auth: msg.sender must be owner or approved
        require(
            msg.sender == from || 
            _tokenApprovals[tokenId] == msg.sender ||
            _operatorApprovals[from][msg.sender],
            "not authorized"
        );
        
        // Clear approval
        delete _tokenApprovals[tokenId];
        
        // Transfer ownership
        _owners[tokenId] = to;
        emit Transfer(from, to, tokenId);
    }
    
    /**
     * @notice Approve an address to manage a specific token
     * @param to Address to approve
     * @param tokenId Token to approve for
     */
    function approve(address to, uint256 tokenId) external {
        address owner = _owners[tokenId];
        require(msg.sender == owner, "not owner");
        
        _tokenApprovals[tokenId] = to;
        emit Approval(owner, to, tokenId);
    }
    
    /**
     * @notice Approve or revoke an operator for all tokens
     * @param operator Address to set approval for
     * @param approved True to approve, false to revoke
     */
    function setApprovalForAll(address operator, bool approved) external {
        _operatorApprovals[msg.sender][operator] = approved;
        emit ApprovalForAll(msg.sender, operator, approved);
    }
    
    /**
     * @notice Get the owner of a token
     * @param tokenId Token to query
     * @return owner Address of the owner
     */
    function ownerOf(uint256 tokenId) external view returns (address owner) {
        owner = _owners[tokenId];
        require(owner != address(0), "token does not exist");
    }
}
