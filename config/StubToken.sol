// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/**
 * @title StubToken
 * @notice Minimal ERC-20-like token for documentation examples
 * @dev Not production-ready - for learning/testing only
 */
contract StubToken {
    // Token balances
    mapping(address => uint256) private _balances;
    
    // Allowances
    mapping(address => mapping(address => uint256)) private _allowances;
    
    // ERC-20 events (Transfer not indexed by value for gas savings)
    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);
    
    /**
     * @notice Mint new tokens to an address
     * @param to Recipient address
     * @param amount Amount to mint
     */
    function mint(address to, uint256 amount) external {
        require(to != address(0), "mint to zero address");
        
        _balances[to] += amount;
        emit Transfer(address(0), to, amount);
    }
    
    /**
     * @notice Transfer tokens to another address
     * @param to Recipient
     * @param amount Amount to transfer
     * @return success True if successful
     */
    function transfer(address to, uint256 amount) external returns (bool) {
        require(to != address(0), "transfer to zero address");
        require(_balances[msg.sender] >= amount, "insufficient balance");
        
        _balances[msg.sender] -= amount;
        _balances[to] += amount;
        emit Transfer(msg.sender, to, amount);
        
        return true;
    }
    
    /**
     * @notice Transfer tokens from one address to another using allowance
     * @param from Address to transfer from
     * @param to Recipient
     * @param amount Amount to transfer
     * @return success True if successful
     */
    function transferFrom(address from, address to, uint256 amount) external returns (bool) {
        require(to != address(0), "transfer to zero address");
        require(_balances[from] >= amount, "insufficient balance");
        require(_allowances[from][msg.sender] >= amount, "insufficient allowance");
        
        _balances[from] -= amount;
        _balances[to] += amount;
        _allowances[from][msg.sender] -= amount;
        
        emit Transfer(from, to, amount);
        return true;
    }
    
    /**
     * @notice Approve an address to spend tokens on your behalf
     * @param spender Address to approve
     * @param amount Amount to approve
     * @return success True if successful
     */
    function approve(address spender, uint256 amount) external returns (bool) {
        _allowances[msg.sender][spender] = amount;
        emit Approval(msg.sender, spender, amount);
        return true;
    }
    
    /**
     * @notice Get the balance of an address
     * @param account Address to query
     * @return balance Token balance
     */
    function balanceOf(address account) external view returns (uint256) {
        return _balances[account];
    }
    
    /**
     * @notice Get the allowance granted by owner to spender
     * @param owner Token owner
     * @param spender Approved spender
     * @return remaining Remaining allowance
     */
    function allowance(address owner, address spender) external view returns (uint256) {
        return _allowances[owner][spender];
    }
}
