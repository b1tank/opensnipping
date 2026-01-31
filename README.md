# OpenSnipping

Lightweight screen recorder and screenshot tool for Linux (cross-platform planned).

---

## ✨ Features

- Screen, window, region, and monitor capture
- Recording with pause/resume
- Screenshots with annotation
- System audio + microphone recording
- HiDPI and fractional scaling support
- Minimal, ephemeral GNOME-style UI

## 🚧 Status

MVP in development. See [plan.md](opensnipping/plan.md) for roadmap.

## 🔧 Development

### Prerequisites

- Node.js 20+
- Rust toolchain
- Linux dependencies:
  ```bash
  sudo apt install libwebkit2gtk-4.1-dev librsvg2-dev libgtk-3-dev
  ```

### Setup

```bash
cd opensnipping
npm install
npm run tauri dev
```

### Testing

```bash
cd opensnipping
npm test                        # UI tests (Vitest)
cd src-tauri && cargo test      # Rust tests
```

## 📝 License

MIT
