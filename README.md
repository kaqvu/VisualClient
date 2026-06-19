# Visual Client App

![Visual Client Logo](https://img.shields.io/badge/Status-Active-brightgreen)
![Tauri Version](https://img.shields.io/badge/Tauri-v2-blue)
![Vue Version](https://img.shields.io/badge/Vue-3-4fc08d)
![Rust](https://img.shields.io/badge/Rust-1.70+-orange)

**Visual Client** is a modern, high-performance Minecraft launcher built with the latest web technologies and Rust. Designed with a focus on beautiful aesthetics, buttery-smooth animations, and unparalleled speed, it offers a premium user experience while keeping resource usage to an absolute minimum.

---

## ✨ Features

- **🎨 Modern Glassmorphism UI:** Stunning, responsive interface built with Vue 3 and modern CSS, featuring dynamic hover effects, smooth micro-animations, and a sleek dark mode.
- **🚀 Ultra-Fast Performance:** Powered by Tauri v2 and Rust, ensuring lightning-fast startup times and minimal RAM usage compared to traditional Electron-based launchers.
- **🔐 Microsoft Authentication:** Seamless integration with Microsoft login for official Minecraft accounts. Securely store and manage multiple accounts.
- **📦 Multi-Instance Management:** Easily create, manage, and launch multiple customized Minecraft instances with isolated directories.
- **🔄 Built-in Auto Updater:** Stay up-to-date effortlessly! Visual Client features a built-in background updater using cryptographic signatures to securely fetch and install the latest releases.
- **🪟 Custom Window Controls:** Custom-designed, frameless title bar with integrated update notifications and native window controls.

## 🛠️ Tech Stack

- **Frontend:** [Vue 3](https://vuejs.org/) (Composition API), [Vite](https://vitejs.dev/), TypeScript, Vanilla CSS (Custom Design System)
- **Backend / Core:** [Rust](https://www.rust-lang.org/), [Tauri v2](https://v2.tauri.app/)
- **State Management:** Vue Reactivity (`ref`, `reactive`)
- **Authentication:** Official Microsoft OAuth flows
- **Updater:** `@tauri-apps/plugin-updater` & GitHub Releases

## ⚙️ Prerequisites

Before you begin, ensure you have met the following requirements:
- **Node.js**: `v18` or higher
- **Package Manager**: `pnpm` (recommended) or `npm`
- **Rust**: Latest stable toolchain (via `rustup`)
- **OS-specific Build Tools**: 
  - *Windows*: Visual Studio C++ Build Tools
  - *macOS*: Xcode Command Line Tools
  - *Linux*: `build-essential`, `libwebkit2gtk-4.1-dev`, `curl`, `wget`, etc. (Check Tauri docs for your distro)

## 🚀 Getting Started

### 1. Clone the repository
```bash
git clone https://github.com/kaqvu/VisualClient.git
cd VisualClient
```

### 2. Install dependencies
```bash
pnpm install
```

### 3. Run the development server
This command starts the Vite dev server and the Tauri Rust backend simultaneously.
```bash
pnpm tauri dev
```

## 📦 Building for Production

To build a standalone executable and installer for your current operating system, run:
```bash
pnpm tauri build
```
The compiled binaries and installers (MSI, NSIS, AppImage, deb, etc.) will be located in the `src-tauri/target/release/bundle/` directory.

### Code Signing & Auto Updates
Visual Client is configured to use Tauri's built-in updater. 
To build the app with updater artifacts (`.sig`, `.zip` archives), ensure your environment variables are configured with your Minisign private key:
```bash
export TAURI_SIGNING_PRIVATE_KEY="your_base64_encoded_private_key"
pnpm tauri build
```
*(Note: A GitHub Actions workflow handles automated building and code signing on release).*

## 🏗️ Project Structure

```text
VisualClient/
├── src/                    # Vue 3 Frontend
│   ├── assets/             # Global CSS, SVGs, and images
│   ├── components/         # Reusable Vue components (UI, Layouts)
│   ├── composables/        # Shared reactive state (e.g., useUpdater.ts)
│   └── views/              # Page views (Library, Settings, Accounts)
├── src-tauri/              # Rust Backend
│   ├── src/                # Rust source code (commands, auth, instances)
│   ├── capabilities/       # Tauri v2 Capabilities & Permissions
│   ├── Cargo.toml          # Rust dependencies
│   └── tauri.conf.json     # Tauri application configuration
├── package.json            # Node.js dependencies
└── vite.config.ts          # Vite build configuration
```

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

---
*Developed with ❤️ by [kaqvu](https://github.com/kaqvu)*
