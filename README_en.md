# InsightFlow User Manual 📚

> English Version | [中文版](README.md)

Welcome to **InsightFlow**! This is a modern, desktop-based time tracking and efficiency monitoring tool designed to help you understand your workflow and maintain deep focus. Through automatic background tracking, an elegant desktop overlay, and a powerful dashboard, InsightFlow puts you in control of your time.

---

## 🌟 Core Features

### 1. Desktop Overlay
The overlay is the visual core of InsightFlow. It stays on top of your desktop to provide real-time feedback on your time:
- **Real-time Activity Monitoring**: Automatically identifies the application currently in the foreground and displays its assigned category (e.g., Development, Entertainment, Productivity).
- **Minimalist Pomodoro Timer**: Click the 🍅 (tomato) icon at the bottom of the overlay to open the Pomodoro panel. It features a built-in **25-minute focus / 5-minute break** loop to help you balance work and rest.
- **Customizable Appearance**: Toggle between Light and Dark modes. Click the settings icon to adjust the overlay's **opacity**.
- **Click-Through Mode**: If the overlay is blocking your code or design work, simply enable "Click-Through" mode using a global shortcut to make it invisible to mouse clicks.

### 2. Smart Self-Learning Classifier
The system comes with built-in rules to automatically categorize the apps you use (like VS Code or Chrome) into corresponding modules.
- **Manual Correction & Learning**: If the system misclassifies an uncommon app (e.g., categorizing it as `Other`), you can find the record in the Dashboard's Activity List, click the dropdown menu on the right, and select the correct category.
- **Automatic Evolution**: Once corrected, InsightFlow automatically "remembers" your choice and adds the app's keyword to its rule base. The next time you open that app, the system will categorize it perfectly.

### 3. Distraction Alerts
Think of it as your personal efficiency assistant. The system features a smart heartbeat algorithm:
- When it detects that you have been continuously using apps in the **"Entertainment"** or **"Social"** categories for more than **15 minutes**, a sleek glassmorphic toast notification will smoothly slide out at the bottom of the overlay.
- It reminds you: *"⚠️ You haven't been focusing for a while. Need a break?"*, helping you break free from the endless scrolling loop.

### 4. Data Dashboard
Click the grid icon on the far left of the overlay to open the powerful local data dashboard:
- **Drag-and-Drop Layout**: The titles of each section (Activity List, Weekly Trends, Module Manager) can be dragged to reorder them. You can freely arrange them according to your preference, and the system will automatically save your layout.
- **Flat Activity Records**: Intuitively displays your application switching history throughout the day in reverse chronological order.
- **Module Manager**: Customize categories by adding new ones (like "Learning" or "Design"), changing their representative colors, and manually managing the process keywords and website domain suffixes that match them.

---

## ⌨️ Shortcuts & Tips

| Action | How to Trigger | Description |
| :--- | :--- | :--- |
| **Drag Overlay** | Click & hold the top bar | Click and hold the app name/timer area to drag the overlay anywhere on your screen. |
| **Toggle Click-Through** | `Ctrl` + `Shift` + `I` | Global shortcut. When enabled, mouse clicks will pass right through the overlay, allowing you to interact with the windows underneath. (Press again to disable) |
| **Hide/Expand Panels** | Click icons/modules | Click various modules and icons to toggle smooth expand/collapse animations, keeping the overlay minimal. |

---

## 🛠️ Installation & Setup (For Developers)

InsightFlow is built with **Rust (Tauri)** and **Vue 3 + TypeScript**. All data is securely stored locally via SQLite (`%APPDATA%/InsightFlow`), ensuring your absolute privacy.

### Prerequisites
1. Install [Node.js](https://nodejs.org/) (v18+ recommended)
2. Install [Rust](https://rustup.rs/) (latest stable version recommended)
3. Ensure your Windows environment has the required C++ build tools (Visual Studio Build Tools) installed.

### Build & Run
```bash
# Navigate to the frontend directory
cd InsightFlow/insight-app

# Install dependencies
npm install

# Run in development mode (with Hot Module Replacement)
npm run tauri dev

# Build for production (.msi / .exe)
npm run tauri build
```

*(Note: During multi-threaded development or initial runs, the underlying SQLite database uses WAL concurrency mode. If a process crash causes a database lock, simply terminate the process in Task Manager and restart.)*

---
*Stay focused, stay creative. —— InsightFlow Team*
