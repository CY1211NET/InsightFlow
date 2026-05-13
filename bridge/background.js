const API_URL = 'http://127.0.0.1:5678/api/web-visit';

let lastSentUrl = '';
let lastSentTime = 0;
const DEBOUNCE_MS = 2000;

async function reportTab(tabId) {
  try {
    const tab = await chrome.tabs.get(tabId);
    if (!tab.url || tab.url.startsWith('chrome://') || tab.url.startsWith('about:') || tab.url.startsWith('edge://')) {
      return;
    }

    const now = Date.now();
    if (tab.url === lastSentUrl && (now - lastSentTime) < DEBOUNCE_MS) {
      return;
    }
    lastSentUrl = tab.url;
    lastSentTime = now;

    await fetch(API_URL, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        url: tab.url,
        title: tab.title || '',
        timestamp: now
      })
    });
  } catch (e) {
    // Silent fail: server may not be running
  }
}

chrome.tabs.onActivated.addListener((activeInfo) => {
  reportTab(activeInfo.tabId);
});

chrome.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
  if (changeInfo.url && tab.active) {
    reportTab(tabId);
  }
});

chrome.windows.onFocusChanged.addListener(async (windowId) => {
  if (windowId === chrome.windows.WINDOW_ID_NONE) return;
  try {
    const [tab] = await chrome.tabs.query({ active: true, windowId });
    if (tab) reportTab(tab.id);
  } catch (e) {
    // Silent fail
  }
});
