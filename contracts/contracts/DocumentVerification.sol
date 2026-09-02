// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/**
 * @title DocumentVerification
 * @notice Lightweight contract for buyer-initiated document verification on Arbitrum Sepolia.
 *         Stores only cryptographic proof — no PII, no files, no business data.
 */
contract DocumentVerification {
    struct Verification {
        bytes32 documentHash;
        bytes32 tradeIdHash;
        bytes32 documentIdHash;
        address verifier;
        uint256 timestamp;
        bool exists;
    }

    // documentHash => Verification
    mapping(bytes32 => Verification) public verifications;

    // Track all verification hashes for enumeration
    bytes32[] public verificationHashes;

    event DocumentVerified(
        bytes32 indexed documentHash,
        bytes32 indexed tradeIdHash,
        bytes32 indexed documentIdHash,
        address verifier,
        uint256 timestamp
    );

    /**
     * @notice Record a document verification on-chain.
     * @param documentHash SHA-256 hash of the actual document bytes (as bytes32).
     * @param tradeIdHash  Keccak-256 hash of the off-chain trade/shipment ID.
     * @param documentIdHash Keccak-256 hash of the off-chain document ID.
     */
    function verifyDocument(
        bytes32 documentHash,
        bytes32 tradeIdHash,
        bytes32 documentIdHash
    ) external {
        require(documentHash != bytes32(0), "Document hash cannot be zero");
        require(!verifications[documentHash].exists, "Document already verified");

        verifications[documentHash] = Verification({
            documentHash: documentHash,
            tradeIdHash: tradeIdHash,
            documentIdHash: documentIdHash,
            verifier: msg.sender,
            timestamp: block.timestamp,
            exists: true
        });

        verificationHashes.push(documentHash);

        emit DocumentVerified(
            documentHash,
            tradeIdHash,
            documentIdHash,
            msg.sender,
            block.timestamp
        );
    }

    /**
     * @notice Check if a document hash has been verified.
     * @param documentHash The hash to check.
     * @return exists Whether the document has been verified.
     * @return verifier The address that verified it.
     * @return timestamp When it was verified.
     */
    function isVerified(bytes32 documentHash)
        external
        view
        returns (bool exists, address verifier, uint256 timestamp)
    {
        Verification memory v = verifications[documentHash];
        return (v.exists, v.verifier, v.timestamp);
    }

    /**
     * @notice Get the total number of verifications recorded.
     */
    function getVerificationCount() external view returns (uint256) {
        return verificationHashes.length;
    }
}
