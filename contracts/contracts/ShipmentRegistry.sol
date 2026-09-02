// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "./OrganizationRegistry.sol";

contract ShipmentRegistry {
    OrganizationRegistry public orgRegistry;

    enum ShipmentStatus { Draft, Pending, UnderReview, Approved, InTransit, Delivered, Closed }

    struct Shipment {
        string shipmentId;
        address exporter;
        address logisticsProvider;
        ShipmentStatus status;
        uint256 timestamp;
    }

    // shipmentId => Shipment
    mapping(string => Shipment) public shipments;

    event ShipmentCreated(string indexed shipmentId, address indexed exporter, uint256 timestamp);
    event ShipmentStatusUpdated(string indexed shipmentId, ShipmentStatus newStatus, address indexed updatedBy, uint256 timestamp);

    modifier onlyVerified() {
        require(orgRegistry.isWalletVerified(msg.sender), "Not a verified organization");
        _;
    }

    constructor(address _orgRegistryAddress) {
        orgRegistry = OrganizationRegistry(_orgRegistryAddress);
    }

    function createShipment(string memory _shipmentId) external onlyVerified {
        require(bytes(shipments[_shipmentId].shipmentId).length == 0, "Shipment already exists");

        shipments[_shipmentId] = Shipment({
            shipmentId: _shipmentId,
            exporter: msg.sender,
            logisticsProvider: address(0),
            status: ShipmentStatus.Pending,
            timestamp: block.timestamp
        });

        emit ShipmentCreated(_shipmentId, msg.sender, block.timestamp);
    }

    function assignLogistics(string memory _shipmentId, address _logistics) external onlyVerified {
        require(shipments[_shipmentId].exporter == msg.sender, "Only exporter can assign logistics");
        shipments[_shipmentId].logisticsProvider = _logistics;
    }

    function updateShipmentStatus(string memory _shipmentId, ShipmentStatus _newStatus) external onlyVerified {
        require(bytes(shipments[_shipmentId].shipmentId).length != 0, "Shipment does not exist");
        
        // Basic access control logic can be added here
        
        shipments[_shipmentId].status = _newStatus;
        emit ShipmentStatusUpdated(_shipmentId, _newStatus, msg.sender, block.timestamp);
    }
}
