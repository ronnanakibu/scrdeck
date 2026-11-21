# ScrDeck

<div align="center">

### Your Android. Your Workspace.

**ScrDeck** is a fast, local-first, open-source graphical companion for the original [scrcpy](https://github.com/Genymobile/scrcpy) project and Android Debug Bridge (**ADB**).

[![Release](https://img.shields.io/badge/release-V1-blue.svg)](https://github.com/ronnanakibu/scrdeck/releases/tag/V1)
[![Platform](https://img.shields.io/badge/platform-Windows%20x64-0078D6.svg)](https://github.com/ronnanakibu/scrdeck)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Built with Tauri](https://img.shields.io/badge/backend-Tauri%20v2%20(Rust)-FFC131.svg)](https://tauri.app)
[![Built with React](https://img.shields.io/badge/frontend-React%20%2B%20TypeScript-61DAFB.svg)](https://react.dev)

</div>

---

## 🚀 Overview

ScrDeck eliminates the friction of remembering and typing complex command-line arguments for `scrcpy` and `adb`. It provides an intuitive, industrial-grade desktop workspace that lets you discover devices, pair wirelessly, adjust mirroring parameters, save custom performance presets, and inspect live diagnostic activity — all with zero telemetry, zero cloud dependencies, and 100% local operation.

---

## 📦 Download & Installation

Prebuilt Windows x64 executables are available directly via official GitHub Releases:

| Asset | Description | Direct Download |
| :--- | :--- | :--- |
| **Windows Installer** | Recommended NSIS installer with desktop shortcuts and auto-updater hooks | [⬇️ **Download ScrDeck-v1.0.0-Setup.exe**](https://github.com/ronnanakibu/scrdeck/releases/download/V1/ScrDeck-v1.0.0-Setup.exe) |
| **Portable Executable** | Standalone portable executable (no installation required) | [⬇️ **Download scrdeck.exe**](https://github.com/ronnanakibu/scrdeck/releases/download/V1/scrdeck.exe) |

> 💡 **Release Hub:** View all release notes and checksums on the [ScrDeck V1 Release Page](https://github.com/ronnanakibu/scrdeck/releases/tag/V1).

> **Note:** On first launch, Windows SmartScreen may show an untrusted warning because the executable is community-built and not signed with an expensive commercial EV certificate. Click **"More info"** &rarr; **"Run anyway"**.

---

## ✨ Features

### 1. Setup Assistant & Automated Toolchain Discovery
- Automatically detects installed `scrcpy` and `adb` binaries in system `PATH` or custom directories.
- In-app managed toolchain installer downloads and configures official verified dependencies with one click.
- Visual, step-by-step guides for enabling **Developer Options** and **USB Debugging** across major OEMs (Samsung, Xiaomi/MIUI, Google Pixel, OnePlus, OPPO, etc.).
- Real-time RSA device authorization verification.

### 2. Device Management & Wireless Debugging
- Manage multiple attached devices simultaneously with distinct status indicators (Authorized, Pending, Offline).
- Native support for **Wireless Debugging** with private pairing code input and automatic port discovery.
- Built-in driver diagnostics with direct access to official manufacturer OEM USB drivers.

### 3. Mirroring Studio
- **Basic Controls**: Display scaling, frame rates (up to 120 FPS), modern video codecs (H.264, H.265, AV1), audio forwarding, and window behavior (always on top, borderless, fullscreen).
- **Power & Screen Optimizations**: Turn off the physical Android screen during mirroring to save battery and prevent burn-in; toggle PC sleep prevention.
- **Advanced Mode**: Configurable display orientation, custom rectangular crop boundaries, maximum bitrates, and direct video recording (MP4 and MKV formats).
- **Custom Arguments**: Transparent argument builder with real-time argument conflict detection, safety validation, and an exact live preview of the command to be executed.
- **Process Isolation**: Starts and stops ScrDeck-managed sessions cleanly without interrupting background ADB tasks.

### 4. Preset Manager
- Bundled profiles: *Balanced*, *Low Latency Gaming*, *High Quality Presentation*, and *Battery Saver*.
- Create, duplicate, modify, and manage custom user presets.
- Portable, schema-versioned JSON import and export for backup or sharing.

### 5. Diagnostics & Activity Timeline
- Chronological event timeline recording connection lifecycles, scrcpy process status, and errors.
- Smart error translation translating obscure ADB exit codes into plain, actionable recovery steps.
- Sanitized diagnostic export with automated redaction of device serial numbers, IP addresses, and user file paths.

### 6. User Experience & Design
- Industrial utility design system featuring Dark, Light, and System themes.
- Lightweight, native desktop performance powered by Tauri v2 and Rust.

---

## 🛠️ Tech Stack & Architecture

- **Desktop Framework:** [Tauri v2](https://tauri.app/) (Rust)
- **Frontend:** [React 18](https://react.dev/), [TypeScript 5](https://www.typescriptlang.org/), [Vite 6](https://vitejs.dev/)
- **Styling:** [Tailwind CSS 3](https://tailwindcss.com/) with custom design tokens
- **State Management:** [Zustand 5](https://github.com/pmndrs/zustand)
- **Icons:** [Lucide React](https://lucide.dev/)
- **Testing:** [Vitest](https://vitest.dev/), [@testing-library/react](https://testing-library.com/)

```
scrdeck/
├── release/              # Prebuilt Windows executables (Installer & Portable)
├── src/                  # React + TypeScript frontend application
│   ├── components/       # Reusable UI elements (Sidebar, TopBar, BrandMark)
│   ├── features/         # Functional feature pages (overview, devices, mirroring, presets, etc.)
│   ├── services/         # Tauri IPC service client
│   ├── stores/           # Zustand reactive stores
│   └── types/            # TypeScript domain interfaces
├── src-tauri/            # Rust native backend
│   ├── src/
│   │   ├── commands/     # Tauri IPC commands
│   │   ├── models/       # Data structures & serialization
│   │   ├── parsers/      # ADB & scrcpy output parsers
│   │   ├── process/      # Process lifecycle & job objects
│   │   ├── services/     # ADB management & command builders
│   │   └── storage/      # Local configuration & preset storage
│   └── tauri.conf.json   # Tauri build and window configuration
├── changelogs/           # Release changelogs
├── scripts/              # Build & resource embedding utilities
└── ScrDeck_Product_Build_Spec_v1.0.md # Complete architectural specification
```

---

## 💻 Development & Building from Source

### Prerequisites

- [Node.js](https://nodejs.org/) (v18 or newer)
- [Rust](https://www.rust-lang.org/) (stable toolchain)
- [Python 3](https://www.python.org/) (for Windows PE resource patching)
- Windows 10/11 (64-bit)

### Installation

```bash
# Clone the repository
git clone https://github.com/ronnanakibu/scrdeck.git
cd scrdeck

# Install frontend dependencies
npm install
```

### Running Locally

```bash
# Start frontend and backend in development mode
npm run tauri dev
```

### Running Tests & Typecheck

```bash
# Run unit tests
npm run test

# Run TypeScript typecheck
npm run lint
```

### Building Release Binaries

```bash
# Build frontend and standalone patched Windows executable
npm run build:exe

# Full Tauri build with NSIS installer bundle
npm run build:app
```

The resulting executable will be in `release/` or `src-tauri/target/release/`.

---

## ⚖️ Disclaimer & Attribution

ScrDeck is an independent, open-source graphical companion for **scrcpy** and **ADB**. It provides a graphical interface for configuring, launching, and managing operations performed by these third-party tools.

- Core screen mirroring, video streaming, and device-control functionality are provided by the original **scrcpy** project developed by Genymobile.
- ADB is provided by **Android SDK Platform-Tools** developed by Google.
- ScrDeck does not own, replace, reimplement, or claim authorship of these upstream tools.
- ScrDeck is not affiliated with, endorsed by, or officially supported by Genymobile or Google.
- All trademarks and brand names belong to their respective owners. See [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) for full license details.

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).
