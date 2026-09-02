const { buildModule } = require("@nomicfoundation/hardhat-ignition/modules");

module.exports = buildModule("DocuTradeModule", (m) => {
  // Deploy Organization Registry
  const orgRegistry = m.contract("OrganizationRegistry");

  // Deploy Shipment Registry
  const shipmentRegistry = m.contract("ShipmentRegistry", [orgRegistry]);

  // Deploy Document Registry
  const documentRegistry = m.contract("DocumentRegistry", [orgRegistry, shipmentRegistry]);

  return { orgRegistry, shipmentRegistry, documentRegistry };
});
