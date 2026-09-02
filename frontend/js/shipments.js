/**
 * DocuTrade — Shipments & Form Helpers
 */

window.DocuTrade = window.DocuTrade || {};

DocuTrade.Shipments = (function() {
  
  // Generic helper to parse form to object
  function serializeForm(formElement) {
    const formData = new FormData(formElement);
    const data = {};
    for (let [key, value] of formData.entries()) {
      data[key] = value;
    }
    return data;
  }

  // Example API calls that map to UI
  async function getShipments() {
    const res = await DocuTrade.API.get(`/shipments`);
    return res.shipments || [];
  }

  async function getShipment(id) {
    return await DocuTrade.API.get(`/shipments/${id}`);
  }
  
  async function updateStatus(id, status) {
    return await DocuTrade.API.post(`/shipments/${id}/status`, { status });
  }

  async function addDocument(id, docData) {
    return await DocuTrade.API.post(`/documents/upload`, docData);
  }

  function getStatusBadge(status) {
    const map = {
      'draft': '<span class="badge badge-draft">Draft</span>',
      'documents_pending': '<span class="badge badge-dot badge-pending">Documents Pending</span>',
      'under_review': '<span class="badge badge-dot badge-review">Under Review</span>',
      'approved': '<span class="badge badge-dot badge-approved">Approved</span>',
      'ready_to_ship': '<span class="badge badge-dot badge-approved">Ready to Ship</span>',
      'in_transit': '<span class="badge badge-dot badge-transit">In Transit</span>',
      'delivered': '<span class="badge badge-dot badge-delivered">Delivered</span>',
      'closed': '<span class="badge badge-dot badge-closed">Closed</span>'
    };
    return map[status] || `<span class="badge">${status}</span>`;
  }

  function formatDate(isoString) {
    if (!isoString) return '';
    const d = new Date(isoString);
    return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
  }
  
  function formatCurrency(value) {
    return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(value || 0);
  }

  return {
    serializeForm,
    getShipments,
    getShipment,
    updateStatus,
    addDocument,
    getStatusBadge,
    formatDate,
    formatCurrency
  };
})();
