## 🎯 Screen Recorder + Screenshot Tool

### 🧠 One-Line Definition

> **A lightweight, cross-platform screen recorder and screenshot tool that matches GNOME Screencast’s minimal UX, adds audio and cursor capture for recordings, and supports basic post-capture annotation for screenshots only.**

### Top Principles
- Quickly deliverable MVP without reinventing the wheel
- Use the most popular and familiar technologies
- Feature complete based on the spec below
- Cross-platform support (Linux primary, Windows/macOS supported)
- Minimal, ephemeral UI

### 1. Capture Capabilities (GNOME Screencast Baseline)

| Category | Feature                    |
| -------- | -------------------------- |
| Screen   | Full screen capture        |
| Screen   | Per-monitor capture        |
| Screen   | Region / selection capture |
| Screen   | Window capture             |
| Display  | Wayland + X11 (Linux)      |
| Display  | HiDPI / fractional scaling |

---

### 2. Recording Controls

| Feature             | Support         |
| ------------------- | --------------- |
| Start / Stop        | ✅               |
| Pause / Resume      | ✅               |
| Global hotkeys      | ✅               |
| Recording indicator | ✅               |
| Minimal floating UI | ✅ (GNOME-style) |

---

### 3. Video & Encoding

| Feature                | Support             |
| ---------------------- | ------------------- |
| Formats                | MP4, MKV            |
| Video codec            | H.264               |
| HW acceleration        | VAAPI / NVENC / AMF |
| Software fallback      | ✅                   |
| Stable long recordings | ✅                   |

---

### 4. Audio (Recording Only)

| Feature              | Support |
| -------------------- | ------- |
| System audio capture | ✅       |
| Microphone capture   | ✅       |
| Record both together | ✅       |
| A/V sync             | ✅       |

---

### 5. Mouse Pointer (Recording Only)

| Feature                     | Support  |
| --------------------------- | -------- |
| Cursor visible in recording | ✅        |
| Correct cursor shape        | ✅        |
| HiDPI cursor scaling        | ✅        |
| Cursor toggle               | Optional |

---

### 6. Annotation (📸 Screenshot Only)

| Feature                        | Support      |
| ------------------------------ | ------------ |
| Applies to screenshots only    | ✅            |
| No annotation during recording | ❌ (explicit) |
| Single-color pen               | ✅            |
| Fixed stroke width             | ✅            |
| Draw after capture             | ✅            |
| Clear / undo                   | Basic        |
| Export annotated image         | ✅            |
| No advanced tools              | ❌            |

---

### 7. UI / UX Principles

| Principle           | Description                  |
| ------------------- | ---------------------------- |
| GNOME Screenshot UI | Baseline reference           |
| Ephemeral UI        | Appears only during capture  |
| Minimal controls    | No settings clutter          |
| Mode-based          | Screenshot / Record          |
| Annotation mode     | Separate post-capture screen |

---

### 8. Cross-Platform Scope

| Platform               | Priority  |
| ---------------------- | --------- |
| Linux (Ubuntu / GNOME) | Primary   |
| Windows                | Supported |
| macOS                  | Supported |

---

### 9. Explicit Non-Goals

| Feature                 | Status |
| ----------------------- | ------ |
| Video annotation        | ❌      |
| Live recording overlays | ❌      |
| Video editing           | ❌      |
| Streaming               | ❌      |
| Advanced image editor   | ❌      |

---
