/**
 * DocuTrade — Authentication Service
 * Handles user login, registration, and session state.
 */

window.DocuTrade = window.DocuTrade || {};

DocuTrade.Auth = (function() {
  
  async function login(email, password) {
    const res = await DocuTrade.API.post('/auth/login', { email, password });
    if (res.token && res.user) {
      localStorage.setItem('dt_token', res.token);
      localStorage.setItem('dt_user', JSON.stringify(res.user));
      DocuTrade.Guards.redirectBasedOnRole();
    }
    return res;
  }

  async function walletLogin() {
    if (typeof window.ethereum === 'undefined') {
      throw new Error('MetaMask is not installed. Please install it to continue.');
    }
    
    // Request account access
    const accounts = await window.ethereum.request({ method: 'eth_requestAccounts' });
    const address = accounts[0];
    
    // 1. Get Nonce
    const nonceRes = await DocuTrade.API.post('/auth/wallet/nonce', { wallet_address: address });
    
    // 2. Sign Message
    const signature = await window.ethereum.request({
      method: 'personal_sign',
      params: [nonceRes.message, address],
    });
    
    // 3. Verify Signature & Authenticate
    const res = await DocuTrade.API.post('/auth/wallet/verify', {
      wallet_address: address,
      nonce: nonceRes.nonce,
      signature: signature
    });
    
    if (res.token && res.user) {
      localStorage.setItem('dt_token', res.token);
      localStorage.setItem('dt_user', JSON.stringify(res.user));
      DocuTrade.Guards.redirectBasedOnRole();
    }
    
    return res;
  }

  async function register(userData) {
    const res = await DocuTrade.API.post('/auth/register', userData);
    if (res.token && res.user) {
      localStorage.setItem('dt_token', res.token);
      localStorage.setItem('dt_user', JSON.stringify(res.user));
      DocuTrade.Guards.redirectBasedOnRole();
    }
    return res;
  }

  function logout() {
    localStorage.removeItem('dt_token');
    localStorage.removeItem('dt_user');
    window.location.href = 'login.html';
  }

  function getUser() {
    try {
      return JSON.parse(localStorage.getItem('dt_user'));
    } catch (e) {
      return null;
    }
  }

  function isAuthenticated() {
    return !!localStorage.getItem('dt_token');
  }
  
  function requireAuth() {
    return DocuTrade.Guards.requireAuth();
  }

  return {
    login,
    walletLogin,
    register,
    logout,
    getUser,
    isAuthenticated,
    requireAuth
  };
})();
