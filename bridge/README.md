# InsightFlow Bridge

Chrome/Edge browser extension that sends tab URL data to the InsightFlow desktop app.

## Installation

1. Open Chrome or Edge and navigate to `chrome://extensions` (or `edge://extensions`)
2. Enable **Developer mode** (toggle in top-right corner)
3. Click **Load unpacked** and select this `bridge/` directory
4. The InsightFlow Bridge icon should appear in the toolbar

## How It Works

The extension listens for tab switches and page navigations, then sends the URL and page title to the local InsightFlow HTTP server at `http://127.0.0.1:5678/api/web-visit`.

- **Silent failure**: If InsightFlow is not running, the extension does nothing — no errors, no notifications
- **Privacy**: Sensitive URLs (containing `password`, `token`, `bank`, etc.) are filtered on the server side
- **Debounce**: Rapid tab switches are deduplicated (2-second window)

## Verification

With InsightFlow running:

```bash
curl http://127.0.0.1:5678/api/health
# Should return: ok
```
