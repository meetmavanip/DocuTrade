/**
 * DocuTrade — Wallet Service (Web3/Arbitrum)
 * Handles MetaMask connection, network switching, and UI updates for wallet state.
 */

window.DocuTrade = window.DocuTrade || {};

DocuTrade.Wallet = (function() {
  const ARBITRUM_SEPOLIA_ID = '0x66eee'; // 421614 in hex
  let currentAccount = null;

  async function connect() {
    if (typeof window.ethereum === 'undefined') {
      DocuTrade.Notifications?.error('Wallet Error', 'MetaMask is not installed.');
      return null;
    }

    try {
      const accounts = await window.ethereum.request({ method: 'eth_requestAccounts' });
      currentAccount = accounts[0];
      await ensureNetwork();
      updateUI(currentAccount);
      return currentAccount;
    } catch (err) {
      console.error('Wallet connection failed', err);
      if (err.code === 4001) {
        DocuTrade.Notifications?.warning('Connection Rejected', 'Please connect your wallet to interact with the blockchain.');
      }
      return null;
    }
  }

  async function ensureNetwork() {
    if (!window.ethereum) return;
    
    const chainId = await window.ethereum.request({ method: 'eth_chainId' });
    if (chainId !== ARBITRUM_SEPOLIA_ID) {
      try {
        await window.ethereum.request({
          method: 'wallet_switchEthereumChain',
          params: [{ chainId: ARBITRUM_SEPOLIA_ID }],
        });
      } catch (switchError) {
        // This error code indicates that the chain has not been added to MetaMask.
        if (switchError.code === 4902) {
          try {
            await window.ethereum.request({
              method: 'wallet_addEthereumChain',
              params: [
                {
                  chainId: ARBITRUM_SEPOLIA_ID,
                  chainName: 'Arbitrum Sepolia',
                  rpcUrls: ['https://sepolia-rollup.arbitrum.io/rpc'],
                  nativeCurrency: {
                    name: 'Ethereum',
                    symbol: 'ETH',
                    decimals: 18
                  },
                  blockExplorerUrls: ['https://sepolia.arbiscan.io/']
                }
              ],
            });
          } catch (addError) {
            console.error('Failed to add network', addError);
          }
        }
      }
    }
  }

  function updateUI(address) {
    const btn = document.getElementById('walletBtn');
    const addressSpan = document.getElementById('walletAddress');
    if (!btn || !addressSpan) return;

    if (address) {
      const shortAddr = `${address.substring(0, 6)}...${address.substring(address.length - 4)}`;
      addressSpan.textContent = shortAddr;
      btn.classList.add('connected');
    } else {
      addressSpan.textContent = 'Connect Wallet';
      btn.classList.remove('connected');
    }
  }

  // Setup event listeners for MetaMask changes
  function initListeners() {
    if (window.ethereum) {
      window.ethereum.on('accountsChanged', (accounts) => {
        if (accounts.length === 0) {
          currentAccount = null;
          updateUI(null);
          DocuTrade.Notifications?.info('Wallet Disconnected', 'Your wallet has been disconnected.');
        } else {
          currentAccount = accounts[0];
          updateUI(currentAccount);
          DocuTrade.Notifications?.success('Wallet Changed', `Connected to ${currentAccount.substring(0,6)}...`);
        }
      });

      window.ethereum.on('chainChanged', (chainId) => {
        if (chainId !== ARBITRUM_SEPOLIA_ID) {
          DocuTrade.Notifications?.warning('Wrong Network', 'Please switch to Arbitrum Sepolia.');
        } else {
          window.location.reload(); // Recommended by MetaMask on chain change
        }
      });
      
      // Check if already connected on load
      window.ethereum.request({ method: 'eth_accounts' }).then(accounts => {
        if (accounts && accounts.length > 0) {
          currentAccount = accounts[0];
          updateUI(currentAccount);
        }
      });
    }
  }

  return {
    connect,
    initListeners,
    getAccount: () => currentAccount
  };
})();
