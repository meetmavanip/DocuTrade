/**
 * DocuTrade — API Service
 * Handles all communication with the backend Rust Axum server.
 */

window.DocuTrade = window.DocuTrade || {};

DocuTrade.API = (function() {
  const BASE_URL = 'http://localhost:3000/api'; // Corrected to point to the nested axum router
  
  // Get Auth token from local storage
  const getToken = () => localStorage.getItem('dt_token');
  
  // Generic fetch wrapper
  async function request(endpoint, options = {}) {
    const url = `${BASE_URL}${endpoint}`;
    const token = getToken();
    
    const headers = {
      'Content-Type': 'application/json',
      ...options.headers,
    };
    
    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }
    
    const config = {
      ...options,
      headers,
    };
    
    // Auto stringify body if it's an object
    if (config.body && typeof config.body === 'object' && !(config.body instanceof FormData)) {
      config.body = JSON.stringify(config.body);
    }
    
    // If FormData, let browser set content-type (removes boundary error)
    if (config.body instanceof FormData) {
      delete headers['Content-Type'];
    }
    
    try {
      const response = await fetch(url, config);
      
      // Handle 401 Unauthorized globally
      if (response.status === 401) {
        if (!url.includes('/auth/')) {
          localStorage.removeItem('dt_token');
          localStorage.removeItem('dt_user');
          window.location.href = 'login.html';
          throw new Error('Session expired. Please log in again.');
        }
      }
      
      let data;
      const contentType = response.headers.get('content-type');
      if (contentType && contentType.includes('application/json')) {
        data = await response.json();
      } else {
        data = await response.text();
      }
      
      if (!response.ok) {
        let errorMsg = data.message || data.error || `Error ${response.status}: ${response.statusText}`;
        if (response.status === 401) errorMsg = "Your session has expired. Please log in again.";
        else if (response.status === 403) errorMsg = "You are not authorized to perform this action.";
        else if (response.status === 413) errorMsg = "File is too large.";
        else if (response.status === 415) errorMsg = "Unsupported document type.";
        else if (response.status >= 500) errorMsg = "Server error while processing the request.";
        throw new Error(errorMsg);
      }
      
      return data;
    } catch (error) {
      console.error(`[DocuTrade API Error] ${endpoint}: ${error.message}`);
      if (error.name === 'TypeError' && error.message === 'Failed to fetch') {
        throw new Error("Cannot connect to DocuTrade backend. Check that the backend server is running.");
      }
      throw error;
    }
  }
  


  return {
    get: (endpoint, options) => request(endpoint, { ...options, method: 'GET' }),
    post: (endpoint, body, options) => request(endpoint, { ...options, method: 'POST', body }),
    put: (endpoint, body, options) => request(endpoint, { ...options, method: 'PUT', body }),
    delete: (endpoint, options) => request(endpoint, { ...options, method: 'DELETE' }),
    request,
    
    // Health check
    ping: () => request('/health'),
  };
})();
