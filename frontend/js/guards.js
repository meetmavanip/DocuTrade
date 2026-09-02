// Frontend Route Guards for DocuTrade

window.DocuTrade = window.DocuTrade || {};
window.DocuTrade.Guards = {
    
    _getRole: function() {
        const userStr = localStorage.getItem('dt_user');
        if (userStr) {
            try {
                const user = JSON.parse(userStr);
                if (user && user.role) return user.role.toUpperCase();
            } catch (e) {}
        }

        const token = localStorage.getItem('dt_token');
        if (!token) return null;
        
        try {
            const base64Url = token.split('.')[1];
            if (!base64Url) return null;
            const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');
            const padded = base64.padEnd(base64.length + (4 - base64.length % 4) % 4, '=');
            const jsonPayload = decodeURIComponent(atob(padded).split('').map(function(c) {
                return '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2);
            }).join(''));
            const payload = JSON.parse(jsonPayload);
            return payload.role ? payload.role.toUpperCase() : null;
        } catch (e) {
            console.error('Invalid token format', e);
            return null;
        }
    },

    requireAuth: function() {
        const token = localStorage.getItem('dt_token');
        if (!token) {
            window.location.href = 'login.html';
            return false;
        }
        return true;
    },

    requireSeller: function() {
        if (!this.requireAuth()) return false;
        
        const role = this._getRole();
        if (role !== 'SELLER') {
            console.warn('Access denied: Seller role required');
            if (role === 'BUYER') {
                window.location.href = 'buyer-dashboard.html';
            } else {
                window.location.href = 'login.html';
            }
            return false;
        }
        return true;
    },

    requireBuyer: function() {
        if (!this.requireAuth()) return false;
        
        const role = this._getRole();
        if (role !== 'BUYER') {
            console.warn('Access denied: Buyer role required');
            if (role === 'SELLER') {
                window.location.href = 'seller-dashboard.html';
            } else {
                window.location.href = 'login.html';
            }
            return false;
        }
        return true;
    },
    
    redirectBasedOnRole: function() {
        const role = this._getRole();
        if (role === 'SELLER') {
            window.location.href = 'seller-dashboard.html';
        } else if (role === 'BUYER') {
            window.location.href = 'buyer-dashboard.html';
        } else {
            window.location.href = 'login.html';
        }
    }
};
