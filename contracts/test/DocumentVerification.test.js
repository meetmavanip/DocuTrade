const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("DocumentVerification", function () {
  let contract;
  let owner;
  let buyer;

  beforeEach(async function () {
    [owner, buyer] = await ethers.getSigners();
    const DocumentVerification = await ethers.getContractFactory("DocumentVerification");
    contract = await DocumentVerification.deploy();
    await contract.waitForDeployment();
  });

  describe("verifyDocument", function () {
    it("should record a document verification", async function () {
      const docHash = ethers.keccak256(ethers.toUtf8Bytes("test-document-sha256-hash"));
      const tradeHash = ethers.keccak256(ethers.toUtf8Bytes("trade-id-123"));
      const docIdHash = ethers.keccak256(ethers.toUtf8Bytes("doc-id-456"));

      await expect(contract.connect(buyer).verifyDocument(docHash, tradeHash, docIdHash))
        .to.emit(contract, "DocumentVerified")
        .withArgs(docHash, tradeHash, docIdHash, buyer.address, (v) => v > 0);

      const [exists, verifier, timestamp] = await contract.isVerified(docHash);
      expect(exists).to.be.true;
      expect(verifier).to.equal(buyer.address);
      expect(timestamp).to.be.greaterThan(0);
    });

    it("should prevent duplicate verification", async function () {
      const docHash = ethers.keccak256(ethers.toUtf8Bytes("test-document"));
      const tradeHash = ethers.keccak256(ethers.toUtf8Bytes("trade-id"));
      const docIdHash = ethers.keccak256(ethers.toUtf8Bytes("doc-id"));

      await contract.connect(buyer).verifyDocument(docHash, tradeHash, docIdHash);

      await expect(
        contract.connect(buyer).verifyDocument(docHash, tradeHash, docIdHash)
      ).to.be.revertedWith("Document already verified");
    });

    it("should reject zero hash", async function () {
      const zeroHash = ethers.ZeroHash;
      const tradeHash = ethers.keccak256(ethers.toUtf8Bytes("trade-id"));
      const docIdHash = ethers.keccak256(ethers.toUtf8Bytes("doc-id"));

      await expect(
        contract.connect(buyer).verifyDocument(zeroHash, tradeHash, docIdHash)
      ).to.be.revertedWith("Document hash cannot be zero");
    });

    it("should track verification count", async function () {
      expect(await contract.getVerificationCount()).to.equal(0);

      const docHash1 = ethers.keccak256(ethers.toUtf8Bytes("doc1"));
      const docHash2 = ethers.keccak256(ethers.toUtf8Bytes("doc2"));
      const tradeHash = ethers.keccak256(ethers.toUtf8Bytes("trade"));
      const docIdHash = ethers.keccak256(ethers.toUtf8Bytes("docid"));

      await contract.connect(buyer).verifyDocument(docHash1, tradeHash, docIdHash);
      expect(await contract.getVerificationCount()).to.equal(1);

      await contract.connect(owner).verifyDocument(docHash2, tradeHash, docIdHash);
      expect(await contract.getVerificationCount()).to.equal(2);
    });
  });

  describe("isVerified", function () {
    it("should return false for unverified documents", async function () {
      const docHash = ethers.keccak256(ethers.toUtf8Bytes("nonexistent"));
      const [exists, verifier, timestamp] = await contract.isVerified(docHash);
      expect(exists).to.be.false;
      expect(verifier).to.equal(ethers.ZeroAddress);
      expect(timestamp).to.equal(0);
    });
  });
});
