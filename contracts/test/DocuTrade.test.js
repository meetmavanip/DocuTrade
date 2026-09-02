const { expect } = require("chai");

describe("DocuTrade Smart Contracts", function () {
  let orgRegistry, shipmentRegistry, documentRegistry;
  let admin, exporter, logistics, inspector;

  before(async function () {
    [admin, exporter, logistics, inspector] = await ethers.getSigners();
    
    const OrgRegistry = await ethers.getContractFactory("OrganizationRegistry");
    orgRegistry = await OrgRegistry.deploy();

    const ShipmentRegistry = await ethers.getContractFactory("ShipmentRegistry");
    shipmentRegistry = await ShipmentRegistry.deploy(orgRegistry.target);

    const DocumentRegistry = await ethers.getContractFactory("DocumentRegistry");
    documentRegistry = await DocumentRegistry.deploy(orgRegistry.target, shipmentRegistry.target);
  });

  describe("Organization Registry", function () {
    it("Should register a new organization", async function () {
      await orgRegistry.registerOrganization(exporter.address, "org-1", "ABC Exports", 1);
      const isVerified = await orgRegistry.isWalletVerified(exporter.address);
      expect(isVerified).to.be.true;
    });
  });

  describe("Shipment Registry", function () {
    it("Should create a shipment", async function () {
      await shipmentRegistry.connect(exporter).createShipment("EXP-001");
      const shipment = await shipmentRegistry.shipments("EXP-001");
      expect(shipment.exporter).to.equal(exporter.address);
    });
  });

  describe("Document Registry", function () {
    it("Should anchor a document", async function () {
      await documentRegistry.connect(exporter).anchorDocument("doc-1", "EXP-001", "invoice", "Qm...", "hash123");
      const isValid = await documentRegistry.verifyDocumentHash("doc-1", "hash123");
      expect(isValid).to.be.true;
    });
  });
});
