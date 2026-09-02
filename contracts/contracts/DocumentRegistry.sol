// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "./OrganizationRegistry.sol";
import "./ShipmentRegistry.sol";

contract DocumentRegistry {
    OrganizationRegistry public orgRegistry;
    ShipmentRegistry public shipmentRegistry;

    enum DocumentStatus { Pending, Approved, Rejected }

    struct Document {
        string documentId;
        string shipmentId;
        string docType;
        string ipfsCid;
        string sha256Hash;
        address uploader;
        DocumentStatus status;
        uint256 timestamp;
    }

    // documentId => Document
    mapping(string => Document) public documents;

    event DocumentAnchored(string indexed documentId, string indexed shipmentId, string sha256Hash, string ipfsCid, address indexed uploader);
    event DocumentStatusUpdated(string indexed documentId, DocumentStatus status, address indexed inspector);

    modifier onlyVerified() {
        require(orgRegistry.isWalletVerified(msg.sender), "Not a verified organization");
        _;
    }

    constructor(address _orgRegistryAddress, address _shipmentRegistryAddress) {
        orgRegistry = OrganizationRegistry(_orgRegistryAddress);
        shipmentRegistry = ShipmentRegistry(_shipmentRegistryAddress);
    }

    function anchorDocument(
        string memory _documentId,
        string memory _shipmentId,
        string memory _docType,
        string memory _ipfsCid,
        string memory _sha256Hash
    ) external onlyVerified {
        require(bytes(documents[_documentId].documentId).length == 0, "Document already exists");

        documents[_documentId] = Document({
            documentId: _documentId,
            shipmentId: _shipmentId,
            docType: _docType,
            ipfsCid: _ipfsCid,
            sha256Hash: _sha256Hash,
            uploader: msg.sender,
            status: DocumentStatus.Pending,
            timestamp: block.timestamp
        });

        emit DocumentAnchored(_documentId, _shipmentId, _sha256Hash, _ipfsCid, msg.sender);
    }

    function reviewDocument(string memory _documentId, DocumentStatus _status) external onlyVerified {
        // Here we could enforce only "Inspector" role
        documents[_documentId].status = _status;
        emit DocumentStatusUpdated(_documentId, _status, msg.sender);
    }

    function verifyDocumentHash(string memory _documentId, string memory _hash) external view returns (bool) {
        require(bytes(documents[_documentId].documentId).length != 0, "Document does not exist");
        return keccak256(abi.encodePacked(documents[_documentId].sha256Hash)) == keccak256(abi.encodePacked(_hash));
    }
}
