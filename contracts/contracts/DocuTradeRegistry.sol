// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "./OrganizationRegistry.sol";
import "./ShipmentRegistry.sol";
import "./DocumentRegistry.sol";

contract DocuTradeRegistry {
    OrganizationRegistry public orgRegistry;
    ShipmentRegistry public shipmentRegistry;
    DocumentRegistry public documentRegistry;

    constructor() {
        orgRegistry = new OrganizationRegistry();
        shipmentRegistry = new ShipmentRegistry(address(orgRegistry));
        documentRegistry = new DocumentRegistry(address(orgRegistry), address(shipmentRegistry));
    }

    function getOrganizationRegistryAddress() external view returns (address) {
        return address(orgRegistry);
    }

    function getShipmentRegistryAddress() external view returns (address) {
        return address(shipmentRegistry);
    }

    function getDocumentRegistryAddress() external view returns (address) {
        return address(documentRegistry);
    }
}
