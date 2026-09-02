// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

contract OrganizationRegistry {
    address public admin;
    
    enum OrgType { None, Exporter, Buyer, Logistics, Inspector, Customs, Platform }

    struct Organization {
        string orgId;      // UUID from off-chain DB
        string name;
        OrgType orgType;
        bool isVerified;
        bool isActive;
    }

    // Mapping from blockchain address to Organization
    mapping(address => Organization) public organizations;
    
    event OrganizationRegistered(address indexed wallet, string orgId, string name, OrgType orgType);
    event OrganizationVerified(address indexed wallet);
    
    modifier onlyAdmin() {
        require(msg.sender == admin, "Only admin can perform this action");
        _;
    }

    constructor() {
        admin = msg.sender;
    }

    function registerOrganization(
        address wallet, 
        string memory _orgId, 
        string memory _name, 
        OrgType _orgType
    ) external onlyAdmin {
        require(!organizations[wallet].isActive, "Organization already registered");
        
        organizations[wallet] = Organization({
            orgId: _orgId,
            name: _name,
            orgType: _orgType,
            isVerified: true, // Auto-verify since admin is registering
            isActive: true
        });

        emit OrganizationRegistered(wallet, _orgId, _name, _orgType);
        emit OrganizationVerified(wallet);
    }

    function isWalletVerified(address wallet) external view returns (bool) {
        return organizations[wallet].isVerified && organizations[wallet].isActive;
    }

    function getOrganization(address wallet) external view returns (Organization memory) {
        return organizations[wallet];
    }
}
