/**
 * DocuTrade — Blockchain Document Verification
 * Handles MetaMask interaction for buyer-initiated document verification on Arbitrum Sepolia.
 */

window.DocuTrade = window.DocuTrade || {};

DocuTrade.BlockchainVerify = (function() {
  // Arbitrum Sepolia configuration
  const ARBITRUM_SEPOLIA_CHAIN_ID = '0x66eee'; // 421614 in hex
  const ARBITRUM_SEPOLIA_CHAIN_ID_DEC = 421614;
  const BLOCK_EXPLORER_URL = 'https://sepolia.arbiscan.io';

  let CONTRACT_ADDRESS = window.DOCUTRADE_VERIFICATION_CONTRACT || null;

  // ABI for DocumentVerification.sol — only the verifyDocument function and event
  const CONTRACT_ABI = [
    {
      "inputs": [
        { "name": "documentHash", "type": "bytes32" },
        { "name": "tradeIdHash", "type": "bytes32" },
        { "name": "documentIdHash", "type": "bytes32" }
      ],
      "name": "verifyDocument",
      "outputs": [],
      "stateMutability": "nonpayable",
      "type": "function"
    },
    {
      "anonymous": false,
      "inputs": [
        { "indexed": true, "name": "documentHash", "type": "bytes32" },
        { "indexed": true, "name": "tradeIdHash", "type": "bytes32" },
        { "indexed": true, "name": "documentIdHash", "type": "bytes32" },
        { "indexed": false, "name": "verifier", "type": "address" },
        { "indexed": false, "name": "timestamp", "type": "uint256" }
      ],
      "name": "DocumentVerified",
      "type": "event"
    }
  ];

  /**
   * Convert a hex SHA-256 hash string (64 chars) to bytes32.
   * SHA-256 produces 32 bytes = 64 hex chars = bytes32.
   */
  function sha256HexToBytes32(hexStr) {
    // Remove 0x prefix if present
    const clean = hexStr.replace(/^0x/, '');
    if (clean.length !== 64) {
      throw new Error(`Invalid SHA-256 hash length: ${clean.length}, expected 64 hex chars`);
    }
    return '0x' + clean;
  }

  /**
   * Compute keccak256 of a UTF-8 string using the browser's ethereum provider.
   * Returns bytes32 hex string.
   */
  function keccak256(str) {
    // Use ethers-style manual keccak256 from scratch
    // We'll encode the function selector manually
    // For simplicity, we use a basic keccak256 implementation
    // Actually, we can compute it in the browser using SubtleCrypto for SHA-256,
    // but keccak256 needs a different approach.
    // We'll use the ABI encoding approach directly.
    
    // Convert string to UTF-8 bytes then pad to 32 bytes (keccak-256)
    const encoder = new TextEncoder();
    const bytes = encoder.encode(str);
    
    // Use a simple keccak256 implementation
    return keccak256Bytes(bytes);
  }

  /**
   * Minimal keccak256 for browser (no external dependencies).
   * Uses the Keccak sponge construction.
   */
  function keccak256Bytes(input) {
    // Keccak-256 constants
    const RC = [
      0x0000000000000001n, 0x0000000000008082n, 0x800000000000808An, 0x8000000080008000n,
      0x000000000000808Bn, 0x0000000080000001n, 0x8000000080008081n, 0x8000000000008009n,
      0x000000000000008An, 0x0000000000000088n, 0x0000000080008009n, 0x000000008000000An,
      0x000000008000808Bn, 0x800000000000008Bn, 0x8000000000008089n, 0x8000000000008003n,
      0x8000000000008002n, 0x8000000000000080n, 0x000000000000800An, 0x800000008000000An,
      0x8000000080008081n, 0x8000000000008080n, 0x0000000080000001n, 0x8000000080008008n
    ];

    const ROTC = [
      1, 3, 6, 10, 15, 21, 28, 36, 45, 55, 2, 14, 27, 41, 56, 8, 25, 43, 62, 18, 39, 61, 20, 44
    ];

    const PILN = [
      10, 7, 11, 17, 18, 3, 5, 16, 8, 21, 24, 4, 15, 23, 19, 13, 12, 2, 20, 14, 22, 9, 6, 1
    ];

    function rotl64(x, n) {
      return ((x << BigInt(n)) | (x >> BigInt(64 - n))) & 0xFFFFFFFFFFFFFFFFn;
    }

    function keccakF(state) {
      for (let round = 0; round < 24; round++) {
        // θ
        const C = new Array(5);
        for (let x = 0; x < 5; x++) {
          C[x] = state[x] ^ state[x + 5] ^ state[x + 10] ^ state[x + 15] ^ state[x + 20];
        }
        for (let x = 0; x < 5; x++) {
          const D = C[(x + 4) % 5] ^ rotl64(C[(x + 1) % 5], 1);
          for (let y = 0; y < 25; y += 5) {
            state[y + x] ^= D;
          }
        }

        // ρ and π
        let last = state[1];
        for (let i = 0; i < 24; i++) {
          const j = PILN[i];
          const temp = state[j];
          state[j] = rotl64(last, ROTC[i]);
          last = temp;
        }

        // χ
        for (let y = 0; y < 25; y += 5) {
          const T = [state[y], state[y + 1], state[y + 2], state[y + 3], state[y + 4]];
          for (let x = 0; x < 5; x++) {
            state[y + x] = T[x] ^ ((~T[(x + 1) % 5]) & T[(x + 2) % 5]);
          }
        }

        // ι
        state[0] ^= RC[round];
      }
    }

    // Rate = 1088 bits = 136 bytes for keccak-256
    const rate = 136;
    const capacity = 64;

    // Pad the input (keccak padding: append 0x01...0x80)
    const inputLen = input.length;
    const blockCount = Math.floor((inputLen + rate) / rate);
    const paddedLen = blockCount * rate;
    const padded = new Uint8Array(paddedLen);
    padded.set(input instanceof Uint8Array ? input : new Uint8Array(input));
    padded[inputLen] ^= 0x01;
    padded[paddedLen - 1] ^= 0x80;

    // Initialize state
    const state = new Array(25).fill(0n);

    // Absorb
    for (let offset = 0; offset < paddedLen; offset += rate) {
      for (let i = 0; i < rate / 8; i++) {
        const idx = offset + i * 8;
        let val = 0n;
        for (let b = 0; b < 8; b++) {
          val |= BigInt(padded[idx + b]) << BigInt(b * 8);
        }
        state[i] ^= val;
      }
      keccakF(state);
    }

    // Squeeze (only need 32 bytes for keccak-256)
    const output = new Uint8Array(32);
    for (let i = 0; i < 4; i++) {
      const val = state[i];
      for (let b = 0; b < 8; b++) {
        output[i * 8 + b] = Number((val >> BigInt(b * 8)) & 0xFFn);
      }
    }

    return '0x' + Array.from(output).map(b => b.toString(16).padStart(2, '0')).join('');
  }

  /**
   * Encode a function call for the smart contract.
   * verifyDocument(bytes32, bytes32, bytes32)
   */
  function encodeFunctionCall(documentHash, tradeIdHash, documentIdHash) {
    // Function selector: first 4 bytes of keccak256("verifyDocument(bytes32,bytes32,bytes32)")
    const sigBytes = new TextEncoder().encode("verifyDocument(bytes32,bytes32,bytes32)");
    const selector = keccak256Bytes(sigBytes).substring(0, 10); // 0x + 8 hex chars = 4 bytes

    // ABI encode: each bytes32 is 32 bytes = 64 hex chars
    const param1 = documentHash.replace(/^0x/, '').padStart(64, '0');
    const param2 = tradeIdHash.replace(/^0x/, '').padStart(64, '0');
    const param3 = documentIdHash.replace(/^0x/, '').padStart(64, '0');

    return selector + param1 + param2 + param3;
  }

  /**
   * Main verification flow.
   * @param {string} documentId - UUID of the document
   * @param {string} documentHash - SHA-256 hex hash of the document (64 chars, no 0x prefix)
   * @param {string} shipmentId - Shipment ID string (e.g. "EXP-IND-2026-...")
   * @param {string} documentIdStr - Document ID string (e.g. "DOC-ABC12345")
   * @returns {Promise<object>} Verification result
   */
  async function verifyOnBlockchain(documentId, documentHash, shipmentId, documentIdStr) {
    // 0. Ensure we have the contract address
    if (!CONTRACT_ADDRESS) {
      try {
        const statusResult = await DocuTrade.API.get('/blockchain/status');
        if (statusResult && statusResult.contract_address) {
          CONTRACT_ADDRESS = statusResult.contract_address;
        }
      } catch (e) {
        console.warn("Could not fetch contract address from backend", e);
      }
    }

    if (!CONTRACT_ADDRESS || CONTRACT_ADDRESS === '0x0000000000000000000000000000000000000000') {
      throw new Error("Contract address is not configured. Please contact the administrator.");
    }

    // 1. Check MetaMask availability
    if (typeof window.ethereum === 'undefined') {
      throw new Error('MetaMask is not installed. Please install MetaMask to verify documents on the blockchain.');
    }

    // 2. Connect wallet
    let accounts;
    try {
      accounts = await window.ethereum.request({ method: 'eth_requestAccounts' });
    } catch (err) {
      if (err.code === 4001) {
        throw new Error('MetaMask connection rejected. Please connect your wallet to continue.');
      }
      throw new Error('Failed to connect to MetaMask: ' + err.message);
    }

    const walletAddress = accounts[0];

    // 3. Check network
    const chainId = await window.ethereum.request({ method: 'eth_chainId' });

    // 4. Require Arbitrum Sepolia
    if (chainId !== ARBITRUM_SEPOLIA_CHAIN_ID) {
      try {
        await window.ethereum.request({
          method: 'wallet_switchEthereumChain',
          params: [{ chainId: ARBITRUM_SEPOLIA_CHAIN_ID }],
        });
      } catch (switchError) {
        if (switchError.code === 4902) {
          try {
            await window.ethereum.request({
              method: 'wallet_addEthereumChain',
              params: [{
                chainId: ARBITRUM_SEPOLIA_CHAIN_ID,
                chainName: 'Arbitrum Sepolia',
                rpcUrls: ['https://sepolia-rollup.arbitrum.io/rpc'],
                nativeCurrency: { name: 'Ethereum', symbol: 'ETH', decimals: 18 },
                blockExplorerUrls: ['https://sepolia.arbiscan.io/']
              }],
            });
          } catch (addError) {
            throw new Error('Please add Arbitrum Sepolia network to MetaMask manually.');
          }
        } else if (switchError.code === 4001) {
          throw new Error('Please switch MetaMask to Arbitrum Sepolia to continue.');
        } else {
          throw new Error('Failed to switch network: ' + switchError.message);
        }
      }
    }

    // Verify we're now on the right network
    const currentChainId = await window.ethereum.request({ method: 'eth_chainId' });
    if (currentChainId !== ARBITRUM_SEPOLIA_CHAIN_ID) {
      throw new Error('Please switch MetaMask to Arbitrum Sepolia.');
    }

    // 5. Prepare hashes for the smart contract
    // documentHash: SHA-256 hex → bytes32 (already 32 bytes)
    const docHashBytes32 = sha256HexToBytes32(documentHash);

    // tradeIdHash: keccak256 of shipment ID string → bytes32
    const tradeIdHashBytes32 = keccak256(shipmentId);

    // documentIdHash: keccak256 of document ID string → bytes32
    const docIdHashBytes32 = keccak256(documentIdStr);

    // 6. Encode the function call
    const data = encodeFunctionCall(docHashBytes32, tradeIdHashBytes32, docIdHashBytes32);

    // 7. Prepare and send transaction via MetaMask
    let txHash;
    try {
      txHash = await window.ethereum.request({
        method: 'eth_sendTransaction',
        params: [{
          from: walletAddress,
          to: CONTRACT_ADDRESS,
          data: data,
          // Let MetaMask estimate gas
        }],
      });
    } catch (err) {
      if (err.code === 4001) {
        throw new Error('CANCELLED: Blockchain transaction cancelled by user.');
      }
      throw new Error('MetaMask transaction failed: ' + err.message);
    }

    // 8. Wait for transaction confirmation
    const receipt = await waitForReceipt(txHash);

    if (!receipt || receipt.status !== '0x1') {
      throw new Error('Transaction failed on-chain. Please try again.');
    }

    const blockNumber = parseInt(receipt.blockNumber, 16);

    // 9. Send verification data to backend
    const backendResult = await DocuTrade.API.post(`/documents/${documentId}/blockchain-verify`, {
      transaction_hash: txHash,
      wallet_address: walletAddress,
      chain_id: ARBITRUM_SEPOLIA_CHAIN_ID_DEC,
      contract_address: CONTRACT_ADDRESS,
      block_number: blockNumber,
      document_hash: documentHash
    });

    return {
      success: true,
      transactionHash: txHash,
      blockNumber: blockNumber,
      walletAddress: walletAddress,
      contractAddress: CONTRACT_ADDRESS,
      chainId: ARBITRUM_SEPOLIA_CHAIN_ID_DEC,
      network: 'Arbitrum Sepolia',
      backendResult: backendResult
    };
  }

  /**
   * Poll for transaction receipt until confirmed.
   */
  async function waitForReceipt(txHash, maxAttempts = 60, intervalMs = 3000) {
    for (let i = 0; i < maxAttempts; i++) {
      try {
        const receipt = await window.ethereum.request({
          method: 'eth_getTransactionReceipt',
          params: [txHash],
        });

        if (receipt) {
          return receipt;
        }
      } catch (err) {
        console.warn('Error polling receipt:', err);
      }

      await new Promise(resolve => setTimeout(resolve, intervalMs));
    }

    throw new Error('Transaction confirmation timed out. The transaction may still be processing.');
  }

  /**
   * Get the Arbiscan URL for a transaction.
   */
  function getExplorerUrl(txHash) {
    return `${BLOCK_EXPLORER_URL}/tx/${txHash}`;
  }

  /**
   * Get contract address (for display).
   */
  function getContractAddress() {
    return CONTRACT_ADDRESS;
  }

  /**
   * Format a shortened address.
   */
  function shortenAddress(addr) {
    if (!addr || addr.length < 10) return addr || '';
    return `${addr.substring(0, 6)}...${addr.substring(addr.length - 4)}`;
  }

  /**
   * Format a shortened hash.
   */
  function shortenHash(hash) {
    if (!hash || hash.length < 14) return hash || '';
    return `${hash.substring(0, 10)}...${hash.substring(hash.length - 4)}`;
  }

  return {
    verifyOnBlockchain,
    getExplorerUrl,
    getContractAddress,
    shortenAddress,
    shortenHash,
    sha256HexToBytes32,
    CHAIN_ID: ARBITRUM_SEPOLIA_CHAIN_ID_DEC,
    NETWORK_NAME: 'Arbitrum Sepolia',
    BLOCK_EXPLORER_URL,
  };
})();
