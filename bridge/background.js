const API_URL = 'http://127.0.0.1:5678/api/web-visit';
const HEARTBEAT_ALARM = 'insightflow-heartbeat';
const HEARTBEAT_INTERVAL_SEC = 30;

// ── Tab tracking state ──────────────────────────────────────────────────────

let activeTabUrl = '';
let activeTabTitle = '';
let activeTabStart = 0; // Date.now() ms

// ── Core: send heartbeat to backend ─────────────────────────────────────────

async function sendHeartbeat(url, title, durationMs) {
  if (!url || url.startsWith('chrome://') || url.startsWith('about:') ||
      url.startsWith('edge://') || url.startsWith('chrome-extension://')) {
    return;
  }
  if (durationMs < 1000) return; // ignore sub-second

  try {
    await fetch(API_URL, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        url,
        title: title || '',
        timestamp: Date.now(),
        duration: durationMs,
      })
    });
  } catch (e) {
    // Silent fail: server may not be running
  }
}

// ── Close current tab's tracking and start a new one ────────────────────────

async function switchToTab(tabId) {
  const now = Date.now();

  // Close previous tab
  if (activeTabUrl && activeTabStart > 0) {
    const duration = now - activeTabStart;
    sendHeartbeat(activeTabUrl, activeTabTitle, duration);
  }

  // Start new tab
  try {
    const tab = await chrome.tabs.get(tabId);
    if (!tab.url || tab.url.startsWith('chrome://') || tab.url.startsWith('about:') ||
        tab.url.startsWith('edge://') || tab.url.startsWith('chrome-extension://')) {
      activeTabUrl = '';
      activeTabTitle = '';
      activeTabStart = 0;
      return;
    }
    activeTabUrl = tab.url;
    activeTabTitle = tab.title || '';
    activeTabStart = now;
  } catch (e) {
    activeTabUrl = '';
    activeTabTitle = '';
    activeTabStart = 0;
  }
}

// ── Event listeners ─────────────────────────────────────────────────────────

chrome.tabs.onActivated.addListener((activeInfo) => {
  switchToTab(activeInfo.tabId);
});

chrome.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
  if (changeInfo.url && tab.active) {
    // URL changed on active tab → close old url, start new
    switchToTab(tabId);
  }
});

chrome.windows.onFocusChanged.addListener(async (windowId) => {
  if (windowId === chrome.windows.WINDOW_ID_NONE) {
    // Browser lost focus → close current tab tracking
    if (activeTabUrl && activeTabStart > 0) {
      const duration = Date.now() - activeTabStart;
      sendHeartbeat(activeTabUrl, activeTabTitle, duration);
    }
    activeTabUrl = '';
    activeTabTitle = '';
    activeTabStart = 0;
    return;
  }
  try {
    const [tab] = await chrome.tabs.query({ active: true, windowId });
    if (tab) switchToTab(tab.id);
  } catch (e) {
    // Silent fail
  }
});

// ── Periodic heartbeat ──────────────────────────────────────────────────────

chrome.alarms.create(HEARTBEAT_ALARM, { periodInMinutes: HEARTBEAT_INTERVAL_SEC / 60 });

chrome.alarms.onAlarm.addListener((alarm) => {
  if (alarm.name !== HEARTBEAT_ALARM) return;
  if (!activeTabUrl || activeTabStart === 0) return;

  const now = Date.now();
  const duration = now - activeTabStart;
  sendHeartbeat(activeTabUrl, activeTabTitle, duration);
  activeTabStart = now; // reset timer, don't close tab
});

// ── Service worker startup: detect current active tab ───────────────────────

chrome.tabs.query({ active: true, currentWindow: true }, (tabs) => {
  if (tabs && tabs[0]) {
    activeTabUrl = tabs[0].url || '';
    activeTabTitle = tabs[0].title || '';
    activeTabStart = Date.now();
  }
});
