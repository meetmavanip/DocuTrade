/**
 * DocuTrade — Toast Notifications
 * Handles showing success/error toasts to the user.
 */

window.DocuTrade = window.DocuTrade || {};

DocuTrade.Notifications = (function() {
  function getContainer() {
    let container = document.getElementById('toastContainer');
    if (!container) {
      container = document.createElement('div');
      container.id = 'toastContainer';
      container.className = 'toast-container';
      document.body.appendChild(container);
    }
    return container;
  }

  function show(title, message, type = 'info', duration = 4000) {
    const container = getContainer();
    const toast = document.createElement('div');
    toast.className = `toast toast-${type}`;
    
    let icon = 'ℹ️';
    if (type === 'success') icon = '✓';
    if (type === 'error') icon = '✕';
    if (type === 'warning') icon = '⚠';

    toast.innerHTML = `
      <div class="toast-icon">${icon}</div>
      <div class="toast-content">
        <div class="toast-title">${title}</div>
        <div class="toast-message">${message}</div>
      </div>
      <button class="toast-close" aria-label="Close">&times;</button>
    `;

    container.appendChild(toast);

    // Trigger reflow for animation
    void toast.offsetWidth;
    toast.classList.add('show');

    // Close on click
    const closeBtn = toast.querySelector('.toast-close');
    closeBtn.addEventListener('click', () => {
      removeToast(toast);
    });

    // Auto close
    if (duration > 0) {
      setTimeout(() => {
        removeToast(toast);
      }, duration);
    }
  }

  function removeToast(toast) {
    if (toast.classList.contains('removing')) return;
    toast.classList.add('removing');
    toast.addEventListener('transitionend', () => {
      toast.remove();
    });
  }

  return {
    show,
    success: (title, msg, duration) => show(title, msg, 'success', duration),
    error: (title, msg, duration) => show(title, msg, 'error', duration),
    warning: (title, msg, duration) => show(title, msg, 'warning', duration),
    info: (title, msg, duration) => show(title, msg, 'info', duration),
  };
})();

// ------------------------------------------------------------------
// DocuTrade — Real-time Notification System
// ------------------------------------------------------------------
DocuTrade.NotificationSystem = (function() {
  let notifications = [];
  let pollInterval = null;
  const POLL_RATE = 15000; // 15 seconds

  async function fetchNotifications() {
    try {
      const response = await fetch('/api/notifications', {
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('docutrade_token')}`
        }
      });
      if (response.ok) {
        const data = await response.json();
        
        // Check for new unread notifications that we haven't seen yet to show toast
        if (notifications.length > 0) {
            const currentIds = new Set(notifications.map(n => n.id));
            const newUnreads = data.filter(n => !n.is_read && !currentIds.has(n.id));
            newUnreads.forEach(n => {
                DocuTrade.Notifications.info(n.title, n.message, 5000);
            });
        }
        
        notifications = data;
        updateUI();
      }
    } catch (e) {
      console.error('Failed to fetch notifications:', e);
    }
  }

  async function markAsRead(id) {
    try {
      await fetch(`/api/notifications/${id}/read`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('docutrade_token')}`
        }
      });
      // Optimistically update
      const notif = notifications.find(n => n.id === id);
      if (notif) notif.is_read = true;
      updateUI();
    } catch (e) {
      console.error('Failed to mark read:', e);
    }
  }

  async function markAllRead() {
    try {
      await fetch('/api/notifications/read-all', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('docutrade_token')}`
        }
      });
      notifications.forEach(n => n.is_read = true);
      updateUI();
    } catch (e) {
      console.error('Failed to mark all read:', e);
    }
  }

  function getIconForType(type) {
    if (type.includes('APPROVED')) return '<span class="notification-icon-success">✓</span>';
    if (type.includes('REJECTED')) return '<span class="notification-icon-error">✕</span>';
    if (type.includes('REQUIRED')) return '<span class="notification-icon-warning">⚠</span>';
    if (type.includes('UPDATED')) return '<span class="notification-icon-info">📦</span>';
    if (type.includes('MESSAGE')) return '<span class="notification-icon-info">💬</span>';
    return '<span class="notification-icon-info">ℹ️</span>';
  }

  function getLinkForNotification(notif) {
      if (notif.type.includes('DOCUMENT')) return `documents.html`;
      if (notif.type.includes('SHIPMENT')) return `shipments.html`;
      if (notif.type.includes('MESSAGE')) return `messages.html`;
      return '#';
  }

  function updateUI() {
    const unreadCount = notifications.filter(n => !n.is_read).length;
    
    // Update Badge
    const badge = document.getElementById('notificationBadge');
    if (badge) {
      if (unreadCount > 0) {
        badge.textContent = unreadCount;
        badge.style.display = 'block';
      } else {
        badge.style.display = 'none';
      }
    }

    // Update Dropdown List
    const list = document.getElementById('notificationDropdownList');
    if (list) {
      if (notifications.length === 0) {
        list.innerHTML = '<div class="notification-empty">You\'re all caught up.<br>No new notifications.</div>';
      } else {
        list.innerHTML = notifications.slice(0, 10).map(n => `
          <div class="notification-item ${n.is_read ? '' : 'unread'}" data-id="${n.id}" data-link="${getLinkForNotification(n)}">
            <div class="notification-item-header">
              <div class="notification-title">
                ${getIconForType(n.type)}
                ${n.title}
              </div>
              <div class="notification-time">${timeAgo(new Date(n.created_at))}</div>
            </div>
            <p class="notification-message">${n.message}</p>
          </div>
        `).join('');

        // Add click handlers
        list.querySelectorAll('.notification-item').forEach(item => {
          item.addEventListener('click', async (e) => {
            const id = item.getAttribute('data-id');
            const link = item.getAttribute('data-link');
            if (item.classList.contains('unread')) {
                await markAsRead(id);
            }
            window.location.href = link;
          });
        });
      }
    }

    // Update Recent Activity (Dashboard)
    const activityFeed = document.getElementById('activityFeed');
    if (activityFeed) {
        if (notifications.length === 0) {
            activityFeed.innerHTML = '<div class="text-sm text-secondary" style="padding: var(--space-4);">No recent activity</div>';
        } else {
            activityFeed.innerHTML = '<div style="display: flex; flex-direction: column;">' + notifications.slice(0, 5).map(n => `
                <div style="display: flex; gap: var(--space-3); padding: var(--space-3) var(--space-4); border-bottom: 1px solid var(--border-glass);">
                    <div style="font-size: 1.1rem; padding-top: 2px;">${getIconForType(n.type)}</div>
                    <div>
                        <div style="color: var(--color-white); font-weight: 500; font-size: var(--text-sm);">${n.title}</div>
                        <div style="color: var(--text-secondary); font-size: var(--text-sm); margin-top: 2px;">${n.message}</div>
                        <div style="color: var(--text-muted); font-size: var(--text-xs); margin-top: 4px;">${timeAgo(new Date(n.created_at))}</div>
                    </div>
                </div>
            `).join('') + '</div>';
        }
    }
  }

  function timeAgo(date) {
    const seconds = Math.floor((new Date() - date) / 1000);
    let interval = seconds / 31536000;
    if (interval > 1) return Math.floor(interval) + " years ago";
    interval = seconds / 2592000;
    if (interval > 1) return Math.floor(interval) + " months ago";
    interval = seconds / 86400;
    if (interval > 1) return Math.floor(interval) + " days ago";
    interval = seconds / 3600;
    if (interval > 1) return Math.floor(interval) + " hours ago";
    interval = seconds / 60;
    if (interval > 1) return Math.floor(interval) + " mins ago";
    if (seconds < 30) return "Just now";
    return Math.floor(seconds) + " secs ago";
  }

  function init() {
    if (!localStorage.getItem('docutrade_token')) return;

    // Toggle Dropdown
    const btn = document.getElementById('notificationBtn');
    const dropdown = document.getElementById('notificationDropdown');
    
    if (btn && dropdown) {
      btn.addEventListener('click', (e) => {
        e.stopPropagation();
        dropdown.classList.toggle('show');
      });

      document.addEventListener('click', (e) => {
        if (!dropdown.contains(e.target) && !btn.contains(e.target)) {
          dropdown.classList.remove('show');
        }
      });
    }

    // Mark All Read
    const markAll = document.getElementById('markAllReadBtn');
    if (markAll) {
      markAll.addEventListener('click', async (e) => {
        e.stopPropagation();
        await markAllRead();
      });
    }

    // Initial Fetch & Poll
    fetchNotifications();
    pollInterval = setInterval(fetchNotifications, POLL_RATE);
  }

  return { init };
})();

document.addEventListener('DOMContentLoaded', () => {
    DocuTrade.NotificationSystem.init();
});

