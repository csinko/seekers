# Seekers

A macOS menu bar app for tracking your Claude Pro/Max usage limits.

![macOS](https://img.shields.io/badge/macOS-10.15+-blue)
![License](https://img.shields.io/badge/license-MIT-green)

## Features

- Shows usage percentage directly in the menu bar
- Tracks both session (5-hour) and weekly (7-day) limits
- Customizable display format and progress indicators
- Optional notifications when approaching limits
- Auto-refresh at configurable intervals
- Native macOS look and feel

## Installation

### Prerequisites

- macOS 10.15 or later
- [Rust](https://rustup.rs/) (for building)
- [Node.js](https://nodejs.org/) 18+ (for building)

### Build from source

```bash
# Clone the repository
git clone https://github.com/csinko/seekers.git
cd seekers

# Install dependencies
npm install

# Build the app
npm run tauri build
```

The built app will be at `src-tauri/target/release/bundle/macos/Seekers.app`

### Development

```bash
npm run tauri dev
```

## Setup

To track your Claude usage, you'll need to get your credentials from claude.ai:

### 1. Get your Session Key

1. Log into [claude.ai](https://claude.ai)
2. Open DevTools (`Cmd + Option + I`)
3. Go to **Application** tab
4. In the left sidebar, expand **Cookies** > `https://claude.ai`
5. Find the cookie named `sessionKey`
6. Copy its value (starts with `sk-ant-sid01-...`)

### 2. Get your Organization ID

1. While on claude.ai, look at the URL
2. It should look like: `https://claude.ai/chat/...` or `https://claude.ai/organizations/xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx/...`
3. If you see `/organizations/` in the URL, copy that UUID
4. If not, open DevTools **Network** tab, refresh, and look for any request to `api.claude.ai` - the org ID will be in the request path

### 3. Enter credentials in Seekers

1. Click the Seekers icon in your menu bar
2. Select **Settings...**
3. Paste your Organization ID and Session Key
4. Settings save automatically

## Configuration

### Menu Bar Display

- **Session only** - Show 5-hour usage
- **Weekly only** - Show 7-day usage
- **Both** - Show both (e.g., "42/18")
- **Higher value** - Show whichever is higher

### Progress Style

Choose from circles, blocks, bar, or dots for the dropdown menu progress indicator.

### Notifications

Set thresholds to get notified when approaching limits.

## Data Storage

Credentials are stored locally at `~/.config/seekers/credentials.json` with secure file permissions (0600 - owner read/write only).

Settings are stored at `~/.config/seekers/settings.json`.

## Disclaimer

This is an unofficial app and is not affiliated with Anthropic. It uses Claude's unofficial API which may change at any time. Use at your own discretion.

## License

MIT
