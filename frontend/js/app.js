window.DocuTrade = window.DocuTrade || {};
window.DocuTrade.App = window.DocuTrade.App || {
  escapeHtml: function(str) {
    if (!str) return '';
    return String(str)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#039;');
  }
};

document.addEventListener('DOMContentLoaded', () => {
  
  // 1. Sidebar Toggle Mobile
  const sidebar = document.getElementById('sidebar');
  const sidebarToggle = document.getElementById('sidebarToggle');
  const sidebarOverlay = document.getElementById('sidebarOverlay');

  function toggleSidebar() {
    if(sidebar) sidebar.classList.toggle('open');
    if(sidebarOverlay) sidebarOverlay.classList.toggle('active');
  }

  if (sidebarToggle) sidebarToggle.addEventListener('click', toggleSidebar);
  if (sidebarOverlay) sidebarOverlay.addEventListener('click', toggleSidebar);

  // 2. Initialize Wallet Listeners
  if (window.DocuTrade && window.DocuTrade.Wallet) {
    window.DocuTrade.Wallet.initListeners();
    const walletBtn = document.getElementById('walletBtn');
    if (walletBtn) {
      walletBtn.addEventListener('click', () => {
        window.DocuTrade.Wallet.connect();
      });
    }
  }

  // 3. Populate User Info (if logged in)
  if (window.DocuTrade && window.DocuTrade.Auth) {
    const updateTopbarUI = (u) => {
      if (!u) return;
      const nameEl = document.getElementById('userName');
      const roleEl = document.getElementById('userRole');
      const avatarEl = document.getElementById('userAvatar');
      
      const firstName = u.first_name || u.firstName || '';
      const lastName = u.last_name || u.lastName || '';
      const fullName = `${firstName} ${lastName}`.trim();
      
      if (nameEl) nameEl.textContent = fullName || u.email || 'Unknown User';
      if (roleEl) roleEl.textContent = (u.role || 'Unknown').charAt(0).toUpperCase() + (u.role || 'unknown').slice(1);
      
      if (avatarEl) {
        let initials = '?';
        if (firstName && lastName) initials = firstName[0] + lastName[0];
        else if (u.email) initials = u.email.substring(0,2);
        avatarEl.textContent = initials.toUpperCase();
      }

      // Global RBAC UI Enforcement
      const userRole = (u.role || '').toUpperCase();
      if (userRole === 'BUYER') {
        document.querySelectorAll('.seller-only').forEach(el => el.style.display = 'none');
        document.querySelectorAll('a[href="create-shipment.html"]').forEach(el => el.style.display = 'none');
      } else if (userRole === 'SELLER') {
        document.querySelectorAll('.buyer-only').forEach(el => el.style.display = 'none');
      }
    };

    const user = window.DocuTrade.Auth.getUser();
    const token = localStorage.getItem('dt_token');
    
    // Redirect to login if on a protected page without auth
    const isPublicPage = window.location.pathname.endsWith('index.html') || window.location.pathname === '/' || window.location.pathname.endsWith('public-verify.html');
    if (!token && !isPublicPage) {
        window.location.href = 'index.html';
        return;
    }

    if (user) {
        updateTopbarUI(user);
    } else if (token && window.DocuTrade.API) {
        // Show loading state for topbar user
        const nameEl = document.getElementById('userName');
        if(nameEl) nameEl.textContent = 'Loading...';
    }

    if (token && window.DocuTrade.API) {
      window.DocuTrade.API.get('/auth/me').then(latestUser => {
        if (latestUser) {
          localStorage.setItem('dt_user', JSON.stringify(latestUser));
          updateTopbarUI(latestUser);
        }
      }).catch(err => {
          console.warn('Could not fetch latest user data:', err);
          if (err.message && err.message.includes('401')) {
              localStorage.removeItem('dt_token');
              localStorage.removeItem('dt_user');
              if (!isPublicPage) window.location.href = 'index.html';
          }
      });
    }
  }

  // 3b. Make Topbar User Clickable
  const userMenus = document.querySelectorAll('.topbar-user');
  userMenus.forEach(menu => {
    menu.addEventListener('click', () => {
      window.location.href = 'profile.html';
    });
  });

  // 4. Highlight active nav item based on current URL
  const currentPath = window.location.pathname.split('/').pop();
  document.querySelectorAll('.nav-item').forEach(item => {
    const href = item.getAttribute('href');
    if (href === currentPath || (currentPath === '' && href === 'dashboard.html')) {
      item.classList.add('active');
    } else {
      item.classList.remove('active'); // let HTML dictate default, but override if matched
    }
  });

});

// ------------------------------------------------------------------
// Global Document Viewer with Blockchain Verification
// ------------------------------------------------------------------
document.addEventListener('DOMContentLoaded', () => {
    if (!document.getElementById('docViewerModal')) {
        const modalHtml = `
          <div class="modal-overlay" id="docViewerModal" style="z-index: 1000;">
            <div class="modal modal-lg" style="max-width: 900px; width: 90%; height: 90vh; display: flex; flex-direction: column;">
              <div class="modal-header" style="flex-shrink: 0;">
                <div style="display: flex; flex-direction: column;">
                    <h3 id="docViewerTitle" style="font-size: 1.25rem;">Document</h3>
                    <div id="docViewerStatus" class="mt-1"></div>
                </div>
                <button class="modal-close" onclick="document.getElementById('docViewerModal').classList.remove('active')">✕</button>
              </div>
              <div class="modal-body" style="flex-grow: 1; overflow: hidden; display: flex; flex-direction: column; padding: 0; background: var(--color-gray-50); position: relative;">
                <div id="docViewerLoading" style="display: flex; align-items: center; justify-content: center; height: 100%; position: absolute; inset: 0;">
                    <div class="text-secondary">Loading document...</div>
                </div>
                <div id="docViewerContent" style="display: none; height: 100%; width: 100%; overflow: auto; align-items: center; justify-content: center;"></div>
                <div id="docViewerError" style="display: none; align-items: center; justify-content: center; height: 100%; padding: 2rem; position: absolute; inset: 0;">
                    <div class="auth-error visible" style="margin: 0; text-align: center;">
                        <strong>Unable to load document.</strong><br>
                        <span id="docViewerErrorMsg"></span>
                    </div>
                </div>
              </div>
              <div class="modal-footer" style="flex-shrink: 0; display: flex; justify-content: space-between; align-items: center;">
                 <div>
                    <a id="docViewerDownloadBtn" href="#" class="btn btn-outline" download>Download</a>
                 </div>
                 <div id="docViewerActions" style="display: flex; gap: 8px;"></div>
              </div>
            </div>
          </div>
          
          <!-- Reject Reason Modal -->
          <div class="modal-overlay" id="rejectReasonModal" style="z-index: 1010;">
            <div class="modal">
              <div class="modal-header"><h3>Reject Document</h3><button class="modal-close" onclick="document.getElementById('rejectReasonModal').classList.remove('active')">✕</button></div>
              <div class="modal-body">
                <p style="margin-bottom: var(--space-4);">Are you sure you want to reject this document?</p>
                <div class="form-group">
                    <label class="form-label form-required">Reason</label>
                    <textarea id="rejectReasonInput" class="form-input" rows="4" placeholder="Enter reason for rejection..." required></textarea>
                </div>
              </div>
              <div class="modal-footer">
                <button class="btn btn-outline" onclick="document.getElementById('rejectReasonModal').classList.remove('active')">Cancel</button>
                <button class="btn btn-danger" id="submitRejectBtn">Reject Document</button>
              </div>
            </div>
          </div>

          <!-- Blockchain Verification Confirmation Modal -->
          <div class="modal-overlay" id="blockchainConfirmModal" style="z-index: 1010;">
            <div class="modal" style="max-width: 520px;">
              <div class="modal-header">
                <h3 id="bcConfirmTitle">⛓ Verify Document on Blockchain</h3>
                <button class="modal-close" onclick="document.getElementById('blockchainConfirmModal').classList.remove('active')">✕</button>
              </div>
              <div class="modal-body">
                <div style="display: flex; flex-direction: column; gap: var(--space-4);">
                  <div class="detail-row">
                    <span class="detail-label">Document</span>
                    <span class="detail-value" id="bcConfirmDocName">—</span>
                  </div>
                  <div class="detail-row">
                    <span class="detail-label">Shipment</span>
                    <span class="detail-value" id="bcConfirmShipment">—</span>
                  </div>
                  <div class="detail-row">
                    <span class="detail-label">SHA-256</span>
                    <div class="hash-display" id="bcConfirmDocHash" style="font-size: 11px; word-break: break-all;">—</div>
                  </div>
                  <div class="detail-row">
                    <span class="detail-label">Current Status</span>
                    <span class="detail-value" id="bcConfirmStatus">—</span>
                  </div>
                  <div class="detail-row">
                    <span class="detail-label">Network</span>
                    <span class="detail-value"><span class="badge badge-blockchain">⛓ Arbitrum Sepolia</span></span>
                  </div>
                  <div style="background: rgba(232,114,42,0.08); border: 1px solid rgba(232,114,42,0.2); border-radius: var(--radius-md); padding: var(--space-3); font-size: var(--text-sm); color: var(--text-secondary);">
                    You are about to record this document verification on the blockchain. This action is permanent and cannot be undone. MetaMask will open for you to confirm the transaction.
                  </div>
                </div>
              </div>
              <div class="modal-footer">
                <button class="btn btn-outline" onclick="document.getElementById('blockchainConfirmModal').classList.remove('active')">Cancel</button>
                <button class="btn btn-primary" id="bcConfirmContinueBtn" style="background: linear-gradient(135deg, #E8722A, #D4621E);">
                  🦊 Continue with MetaMask
                </button>
              </div>
            </div>
          </div>

          <!-- Blockchain Verification Progress Modal -->
          <div class="modal-overlay" id="blockchainProgressModal" style="z-index: 1020;">
            <div class="modal" style="max-width: 480px;">
              <div class="modal-header">
                <h3>⛓ Blockchain Verification</h3>
              </div>
              <div class="modal-body" style="text-align: center; padding: var(--space-8);">
                <div id="bcProgressIcon" style="font-size: 48px; margin-bottom: var(--space-4);">⏳</div>
                <div id="bcProgressTitle" style="font-size: var(--text-lg); font-weight: 600; margin-bottom: var(--space-2);">Processing...</div>
                <div id="bcProgressMessage" style="color: var(--text-secondary); font-size: var(--text-sm);">Waiting for blockchain confirmation...</div>
                <div id="bcProgressDetails" style="margin-top: var(--space-4); display: none;"></div>
              </div>
              <div class="modal-footer" id="bcProgressFooter" style="display: none;">
                <button class="btn btn-primary" onclick="document.getElementById('blockchainProgressModal').classList.remove('active'); window.location.reload();">Done</button>
              </div>
            </div>
          </div>

          <!-- Verification Details Modal -->
          <div class="modal-overlay" id="verificationDetailsModal" style="z-index: 1000;">
            <div class="modal modal-lg" style="max-width: 640px;">
              <div class="modal-header">
                <h3>🔍 Document Verification</h3>
                <button class="modal-close" onclick="document.getElementById('verificationDetailsModal').classList.remove('active')">✕</button>
              </div>
              <div class="modal-body" id="verificationDetailsBody">
                <div class="text-secondary text-center py-4">Loading verification details...</div>
              </div>
              <div class="modal-footer" id="verificationDetailsFooter">
              </div>
            </div>
          </div>
        `;
        document.body.insertAdjacentHTML('beforeend', modalHtml);
    }
});

window.DocuTrade = window.DocuTrade || {};
window.DocuTrade.Viewer = {
    currentDoc: null,
    objectUrl: null,

    async viewDocument(docId, docName, status, docHash, shipmentId, documentIdStr) {
        this.currentDoc = { id: docId, name: docName, status: status, hash: docHash, shipmentId: shipmentId, documentIdStr: documentIdStr };
        
        const modal = document.getElementById('docViewerModal');
        const title = document.getElementById('docViewerTitle');
        const loading = document.getElementById('docViewerLoading');
        const content = document.getElementById('docViewerContent');
        const error = document.getElementById('docViewerError');
        const statusBadge = document.getElementById('docViewerStatus');
        const actions = document.getElementById('docViewerActions');
        const dlBtn = document.getElementById('docViewerDownloadBtn');
        
        modal.classList.add('active');
        title.textContent = docName;
        
        // Status badge with all new statuses
        statusBadge.innerHTML = DocuTrade.Viewer.getStatusBadgeHtml(status);
        
        // Reset view
        loading.style.display = 'flex';
        content.style.display = 'none';
        error.style.display = 'none';
        content.innerHTML = '';
        actions.innerHTML = '';
        
        if (this.objectUrl) {
            URL.revokeObjectURL(this.objectUrl);
            this.objectUrl = null;
        }

        // Setup actions based on Role + Status
        const user = window.DocuTrade.Auth.getUser();
        const role = (user && user.role) ? user.role.toUpperCase() : '';
        
        if (role === 'BUYER') {
            const normalizedStatus = (status || '').toUpperCase().replace('_', ' ');
            if (normalizedStatus === 'PENDING' || normalizedStatus === 'PENDING REVIEW') {
                actions.innerHTML = `
                    <button class="btn btn-danger" onclick="DocuTrade.Viewer.showReject()">❌ Reject</button>
                    <button class="btn btn-success" onclick="DocuTrade.Viewer.showBlockchainConfirmation()" style="background: linear-gradient(135deg, #10B981, #059669);">⛓ Approve & Verify</button>
                `;
            } else if (normalizedStatus === 'APPROVED') {
                actions.innerHTML = `
                    <button class="btn btn-primary" onclick="DocuTrade.Viewer.startBlockchainVerify()" style="background: linear-gradient(135deg, #E8722A, #D4621E);">⛓ Verify on Blockchain</button>
                `;
            } else if (normalizedStatus === 'VERIFIED') {
                actions.innerHTML = `
                    <button class="btn btn-outline" onclick="DocuTrade.Viewer.viewVerification('${docId}')">🔍 View Verification</button>
                `;
            } else if (normalizedStatus === 'BLOCKCHAIN FAILED' || normalizedStatus === 'BLOCKCHAIN_FAILED') {
                actions.innerHTML = `
                    <button class="btn btn-primary" onclick="DocuTrade.Viewer.startBlockchainVerify()" style="background: linear-gradient(135deg, #E8722A, #D4621E);">🔄 Try Again</button>
                `;
            }
        } else if (role === 'SELLER') {
            if (status === 'VERIFIED') {
                actions.innerHTML = `
                    <button class="btn btn-outline" onclick="DocuTrade.Viewer.viewVerification('${docId}')">🔍 View Verification</button>
                `;
            }
        }

        try {
            const token = localStorage.getItem('dt_token');
            const res = await fetch(`http://localhost:3000/api/documents/${docId}/file`, {
                headers: { 'Authorization': `Bearer ${token}` }
            });
            
            if (!res.ok) {
                let msg = 'Failed to fetch document.';
                if (res.status === 403 || res.status === 401) msg = 'You do not have permission to view this document.';
                else if (res.status === 404) msg = 'Document not found.';
                throw new Error(msg);
            }
            
            const contentType = res.headers.get('content-type') || 'application/octet-stream';
            const blob = await res.blob();
            this.objectUrl = URL.createObjectURL(blob);
            
            dlBtn.href = this.objectUrl;
            dlBtn.download = docName || 'document';
            
            loading.style.display = 'none';
            content.style.display = 'flex';
            
            if (contentType.includes('pdf')) {
                content.innerHTML = `<object data="${this.objectUrl}" type="application/pdf" width="100%" height="100%">
                    <p>PDF cannot be displayed. <a href="${this.objectUrl}">Download</a></p>
                </object>`;
            } else if (contentType.includes('image/')) {
                content.innerHTML = `<img src="${this.objectUrl}" style="max-width:100%; max-height:100%; object-fit:contain;">`;
            } else {
                content.innerHTML = `<div class="text-center p-8">
                    <div style="font-size: 48px; margin-bottom: 16px;">📄</div>
                    <div>Preview unavailable for this file type.</div>
                    <a href="${this.objectUrl}" class="btn btn-primary mt-4" download="${docName}">Download File</a>
                </div>`;
            }
            
        } catch (err) {
            loading.style.display = 'none';
            error.style.display = 'flex';
            document.getElementById('docViewerErrorMsg').textContent = err.message;
        }
    },

    /**
     * Step 1: Approve the document first, then show blockchain confirmation.
     */
    async startApproveAndVerify() {
        if (!this.currentDoc) return;
        // DO NOT immediately change status in DB. Skip to blockchain confirmation.
        this.showBlockchainConfirmation();
    },

    /**
     * Start blockchain verification for already-approved documents.
     */
    startBlockchainVerify() {
        if (!this.currentDoc) return;
        this.showBlockchainConfirmation();
    },

    /**
     * Show the blockchain confirmation modal with document details.
     */
    showBlockchainConfirmation() {
        if (!this.currentDoc) return;

        // Close the viewer
        document.getElementById('docViewerModal').classList.remove('active');

        // Populate confirmation modal
        document.getElementById('bcConfirmDocName').textContent = this.currentDoc.name;
        document.getElementById('bcConfirmDocHash').textContent = this.currentDoc.hash || 'Fetching...';
        document.getElementById('bcConfirmShipment').textContent = this.currentDoc.documentIdStr || this.currentDoc.shipmentId || '—';
        
        // Handle "Pending Review" display logic
        let displayStatus = this.currentDoc.status || '';
        if (displayStatus.toUpperCase() === 'PENDING') displayStatus = 'Pending Review';
        document.getElementById('bcConfirmStatus').textContent = displayStatus;
        
        const isPending = (displayStatus.toUpperCase().includes('PENDING'));
        document.getElementById('bcConfirmTitle').textContent = isPending ? 'Approve Document?' : '⛓ Verify Document on Blockchain';
        document.getElementById('bcConfirmContinueBtn').innerHTML = isPending ? '🦊 Continue & Approve' : '🦊 Continue with MetaMask';

        // If we don't have the hash yet, fetch it
        if (!this.currentDoc.hash) {
            DocuTrade.API.get(`/documents/${this.currentDoc.id}/verification`).then(result => {
                if (result.document && result.document.document_hash) {
                    this.currentDoc.hash = result.document.document_hash;
                    document.getElementById('bcConfirmDocHash').textContent = result.document.document_hash;
                }
            }).catch(() => {});
        }

        // Set up the continue button
        document.getElementById('bcConfirmContinueBtn').onclick = () => this.executeBlockchainVerification();

        // Show modal
        document.getElementById('blockchainConfirmModal').classList.add('active');
    },

    /**
     * Execute the actual blockchain verification via MetaMask.
     */
    async executeBlockchainVerification() {
        if (!this.currentDoc || !this.currentDoc.hash) {
            window.DocuTrade.Notifications.error('Error', 'Document hash not available. Please try again.');
            return;
        }

        // Close confirmation modal, show progress modal
        document.getElementById('blockchainConfirmModal').classList.remove('active');
        
        const progressModal = document.getElementById('blockchainProgressModal');
        const progressIcon = document.getElementById('bcProgressIcon');
        const progressTitle = document.getElementById('bcProgressTitle');
        const progressMessage = document.getElementById('bcProgressMessage');
        const progressDetails = document.getElementById('bcProgressDetails');
        const progressFooter = document.getElementById('bcProgressFooter');

        progressModal.classList.add('active');
        progressIcon.textContent = '🦊';
        progressTitle.textContent = 'Waiting for MetaMask...';
        progressMessage.textContent = 'Please confirm the transaction in MetaMask.';
        progressDetails.style.display = 'none';
        progressFooter.style.display = 'none';

        try {
            // Update progress for transaction sent
            const onTxSent = () => {
                progressIcon.textContent = '⏳';
                progressTitle.textContent = 'Transaction Submitted';
                progressMessage.textContent = 'Waiting for blockchain confirmation on Arbitrum Sepolia...';
            };

            // Execute blockchain verification
            const result = await DocuTrade.BlockchainVerify.verifyOnBlockchain(
                this.currentDoc.id,
                this.currentDoc.hash,
                this.currentDoc.shipmentId || '',
                this.currentDoc.documentIdStr || ''
            );

            // Success!
            progressIcon.textContent = '✅';
            progressTitle.textContent = 'Blockchain Verified!';
            progressMessage.textContent = 'Document verification has been permanently recorded on the blockchain.';
            
            const BV = DocuTrade.BlockchainVerify;
            progressDetails.style.display = 'block';
            progressDetails.innerHTML = `
                <div style="background: var(--color-gray-50); border-radius: var(--radius-md); padding: var(--space-3); text-align: left; font-size: var(--text-sm);">
                    <div class="detail-row" style="margin-bottom: var(--space-2);">
                        <span class="detail-label">Network</span>
                        <span class="detail-value">Arbitrum Sepolia</span>
                    </div>
                    <div class="detail-row" style="margin-bottom: var(--space-2);">
                        <span class="detail-label">Transaction</span>
                        <a href="${BV.getExplorerUrl(result.transactionHash)}" target="_blank" class="detail-value mono" style="font-size: 11px; color: var(--color-orange-500);">${BV.shortenHash(result.transactionHash)} ↗</a>
                    </div>
                    <div class="detail-row" style="margin-bottom: var(--space-2);">
                        <span class="detail-label">Block</span>
                        <span class="detail-value mono">${result.blockNumber.toLocaleString()}</span>
                    </div>
                    <div class="detail-row">
                        <span class="detail-label">Wallet</span>
                        <span class="detail-value mono" style="font-size: 11px;">${BV.shortenAddress(result.walletAddress)}</span>
                    </div>
                </div>
            `;

            progressFooter.style.display = 'flex';

        } catch (err) {
            if (err.message && err.message.startsWith('CANCELLED:')) {
                // User cancelled MetaMask
                progressIcon.textContent = '🚫';
                progressTitle.textContent = 'Transaction Cancelled';
                progressMessage.textContent = 'Blockchain transaction cancelled by user. The document remains approved but not blockchain-verified.';
            } else {
                // Error
                progressIcon.textContent = '❌';
                progressTitle.textContent = 'Verification Failed';
                progressMessage.textContent = err.message || 'An error occurred during blockchain verification.';
            }

            progressFooter.style.display = 'flex';
            progressFooter.innerHTML = `
                <button class="btn btn-outline" onclick="document.getElementById('blockchainProgressModal').classList.remove('active')">Close</button>
                <button class="btn btn-primary" onclick="document.getElementById('blockchainProgressModal').classList.remove('active'); DocuTrade.Viewer.showBlockchainConfirmation();">Try Again</button>
            `;
        }
    },

    /**
     * View verification details for a document.
     */
    async viewVerification(docId) {
        // Close any open modals
        document.getElementById('docViewerModal')?.classList.remove('active');

        const modal = document.getElementById('verificationDetailsModal');
        const body = document.getElementById('verificationDetailsBody');
        const footer = document.getElementById('verificationDetailsFooter');

        modal.classList.add('active');
        body.innerHTML = '<div class="text-secondary text-center py-4">Loading verification details...</div>';
        footer.innerHTML = '';

        try {
            const data = await DocuTrade.API.get(`/documents/${docId}/verification`);
            const doc = data.document;
            const bc = data.blockchain;

            let blockchainHtml = '';
            if (bc) {
                const BV = DocuTrade.BlockchainVerify;
                const explorerUrl = BV.getExplorerUrl(bc.transaction_hash);
                const verifiedDate = bc.verified_at ? new Date(bc.verified_at).toLocaleString('en-US', { day: '2-digit', month: 'short', year: 'numeric', hour: '2-digit', minute: '2-digit' }) : '—';

                blockchainHtml = `
                    <div style="display: flex; align-items: center; gap: var(--space-2); margin-bottom: var(--space-4);">
                        <span style="font-size: 24px;">🟢</span>
                        <span style="font-size: var(--text-lg); font-weight: 700; color: var(--color-green-600);">BLOCKCHAIN VERIFIED</span>
                    </div>
                    <div class="detail-row" style="margin-bottom: var(--space-3);">
                        <span class="detail-label">Network</span>
                        <span class="detail-value"><span class="badge badge-blockchain">⛓ ${bc.network}</span></span>
                    </div>
                    <div class="detail-row" style="margin-bottom: var(--space-3);">
                        <span class="detail-label">Chain ID</span>
                        <span class="detail-value mono">${bc.chain_id}</span>
                    </div>
                    <div class="detail-row" style="margin-bottom: var(--space-3);">
                        <span class="detail-label">Contract</span>
                        <span class="detail-value mono" style="font-size: 11px; word-break: break-all;">${bc.contract_address}</span>
                    </div>
                    <div class="detail-row" style="margin-bottom: var(--space-3);">
                        <span class="detail-label">Transaction</span>
                        <a href="${explorerUrl}" target="_blank" class="detail-value mono" style="font-size: 11px; color: var(--color-orange-500); word-break: break-all;">${bc.transaction_hash} ↗</a>
                    </div>
                    <div class="detail-row" style="margin-bottom: var(--space-3);">
                        <span class="detail-label">Block</span>
                        <span class="detail-value mono">${bc.block_number ? bc.block_number.toLocaleString() : '—'}</span>
                    </div>
                    <div class="detail-row" style="margin-bottom: var(--space-3);">
                        <span class="detail-label">Verified By</span>
                        <span class="detail-value mono" style="font-size: 11px; word-break: break-all;">${bc.wallet_address}</span>
                    </div>
                    <div class="detail-row">
                        <span class="detail-label">Verified At</span>
                        <span class="detail-value">${verifiedDate}</span>
                    </div>
                `;

                footer.innerHTML = `
                    <a href="${explorerUrl}" target="_blank" class="btn btn-outline">View on Block Explorer ↗</a>
                    <button class="btn btn-primary" onclick="document.getElementById('verificationDetailsModal').classList.remove('active')">Done</button>
                `;
            } else {
                blockchainHtml = `
                    <div style="display: flex; align-items: center; gap: var(--space-2); margin-bottom: var(--space-4);">
                        <span style="font-size: 24px;">🟡</span>
                        <span style="font-size: var(--text-lg); font-weight: 700; color: var(--text-secondary);">NOT BLOCKCHAIN VERIFIED</span>
                    </div>
                    <div class="text-secondary text-sm">This document has not been verified on the blockchain yet.</div>
                `;

                footer.innerHTML = `
                    <button class="btn btn-primary" onclick="document.getElementById('verificationDetailsModal').classList.remove('active')">Close</button>
                `;
            }

            body.innerHTML = `
                <div style="display: flex; flex-direction: column; gap: var(--space-4);">
                    <div style="border-bottom: 1px solid var(--border-light); padding-bottom: var(--space-4);">
                        <h4 style="font-size: var(--text-base); font-weight: 600; margin-bottom: var(--space-3);">📄 Document</h4>
                        <div class="detail-row" style="margin-bottom: var(--space-2);">
                            <span class="detail-label">Name</span>
                            <span class="detail-value">${doc.filename}</span>
                        </div>
                        <div class="detail-row" style="margin-bottom: var(--space-2);">
                            <span class="detail-label">Type</span>
                            <span class="detail-value">${doc.document_type}</span>
                        </div>
                        <div class="detail-row" style="margin-bottom: var(--space-2);">
                            <span class="detail-label">Document Hash</span>
                            <div class="hash-display" style="font-size: 11px; word-break: break-all;">${doc.document_hash}</div>
                        </div>
                        <div class="detail-row">
                            <span class="detail-label">Database Status</span>
                            ${DocuTrade.Viewer.getStatusBadgeHtml(doc.database_status)}
                        </div>
                    </div>
                    <div>
                        <h4 style="font-size: var(--text-base); font-weight: 600; margin-bottom: var(--space-3);">⛓ Blockchain Verification</h4>
                        ${blockchainHtml}
                    </div>
                </div>
            `;

        } catch (err) {
            body.innerHTML = `<div class="auth-error visible" style="margin: 0; text-align: center;"><strong>Error loading verification details.</strong><br>${err.message}</div>`;
            footer.innerHTML = `<button class="btn btn-primary" onclick="document.getElementById('verificationDetailsModal').classList.remove('active')">Close</button>`;
        }
    },

    /**
     * Get a status badge HTML for any document status.
     */
    getStatusBadgeHtml(status) {
        const s = (status || '').toUpperCase();
        switch (s) {
            case 'PENDING':
                return '<span class="badge badge-dot badge-pending">🟡 Pending Review</span>';
            case 'APPROVED':
                return '<span class="badge badge-dot badge-approved">✅ Approved</span>';
            case 'REJECTED':
                return '<span class="badge badge-dot badge-error" style="color: var(--color-red-600);">🔴 Rejected</span>';
            case 'VERIFIED':
                return '<span class="badge badge-dot badge-approved" style="color: var(--color-green-600);">🟢 Blockchain Verified</span>';
            case 'BLOCKCHAIN_PENDING':
                return '<span class="badge badge-dot badge-pending">🟡 Blockchain Verification Pending</span>';
            case 'BLOCKCHAIN_FAILED':
                return '<span class="badge badge-dot badge-error" style="color: var(--color-red-600);">🔴 Blockchain Verification Failed</span>';
            case 'BLOCKCHAIN_REJECTED':
                return '<span class="badge badge-dot badge-error" style="color: var(--color-red-600);">🔴 Blockchain Rejected</span>';
            case 'SUPERSEDED':
                return '<span class="badge badge-dot badge-superseded">Superseded</span>';
            default:
                return `<span class="badge">${status}</span>`;
        }
    },

    showReject() {
        if (!this.currentDoc) return;
        document.getElementById('rejectReasonInput').value = '';
        document.getElementById('rejectReasonModal').classList.add('active');
        document.getElementById('submitRejectBtn').onclick = () => this.submitReject();
    },
    
    async submitReject() {
        const reason = document.getElementById('rejectReasonInput').value;
        if (!reason) { 
            window.DocuTrade.Notifications.warning('Required', 'Rejection reason is required.');
            return; 
        }
        
        try {
            await window.DocuTrade.API.post(`/documents/${this.currentDoc.id}/reject`, { reason });
            window.DocuTrade.Notifications.success('Rejected', 'Document rejected');
            document.getElementById('rejectReasonModal').classList.remove('active');
            document.getElementById('docViewerModal').classList.remove('active');
            window.location.reload();
        } catch (e) {
            window.DocuTrade.Notifications.error('Error', e.message);
        }
    }
};
