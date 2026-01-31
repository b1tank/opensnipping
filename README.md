# OpenSnipping

A lightweight screen recorder and screenshot tool for Linux (with cross-platform support planned).

---

## ✨ Features

- Screen, window, region, and monitor capture
- Recording with pause/resume
- Screenshot with simple annotation
- System audio + microphone recording
- HiDPI / fractional scaling support
- Minimal, ephemeral GNOME-style UI

## 🚧 Status

MVP in development. See [plan.md](plan.md) for roadmap.

## 🔧 Development

### Prerequisites

- Node.js 20+
- Rust toolchain
- Linux system dependencies:
  ```bash
  sudo apt install libwebkit2gtk-4.1-dev librsvg2-dev libgtk-3-dev
  ```

### Setup

```bash
npm install
npm run tauri dev
```

### Testing

```bash
npm test           # UI tests (Vitest)
cd src-tauri && cargo test  # Rust tests
```

## 📝 License

MIT
