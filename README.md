# Hyprside

**Hyprside** is a next-generation immutable Linux operating system built around modern UI principles and GPU acceleration.
It combines native performance, deep system integration, and full user customization — without sacrificing determinism or security.

---

## 🧭 Philosophy

Hyprside is a love letter to the Linux kernel — a demonstration of what it can truly become.

- Takes inspiration from mobile operating systems like Android and iOS, and from Apple’s UX/UI philosophy
- The base system is **immutable**, delivered as a **SquashFS image**
- User configuration is stored in the **HyprRegistry**, not traditional dotfiles
- The entire UI stack is powered by **HyprUI**, a GPU-native Rust framework
- The compositor is a fork of **Hyprland**, adapted for non-technical users
- Designed to be **daemonless** wherever possible — fewer background processes, more CPU for your apps

---

## 🧩 (Planned) Core Components

| Component | Description |
|------------|-------------|
| **HyprUI** | Declarative immediate-mode UI framework (Rust + Skia + Clay) |
| **HyprDE** | Complete desktop environment built with HyprUI |
| **HyprTheme** | System-wide theming and visual identity manager that works consistently across toolkits and apps |
| **Shift** | A replacement for the Linux TTY system, enabling smooth transitions between screens |
| **Hyprinit** | Minimal and purpose-built init system designed specifically for Hyprside |
| **Hyprpacker** | Unified build system for the kernel, image, and initrd |
| **Kernel Bombproof** | Hardened Linux kernel fork with patches to prevent UI freezes under heavy workloads |

---

## ⚙️ Build Pipeline Overview

Hyprside is fully built via **Hyprpacker**:

1. `manifest.toml` defines the system components
2. Hyprpacker compiles the **kernel**, **initrd**, and all **packages**
3. The result is an immutable SquashFS system image
4. You can test it directly in QEMU with UEFI boot

```bash
hyprpacker vm run
````

This command performs a full build and automatically boots the OS.

---

## 🧱 Repository Structure

```
hyprside/
 ├── packages/
 │   ├── hyprpacker/           # Build tool
 │   ├── hyprinit/             # Stage-1 init
 │   ├── hyprui/               # UI framework
 │   ├── hyprde/               # Desktop environment
 │   └── ...
 ├── system_root/              # Base system contents (mounted as SquashFS)
 ├── build/                    # Generated artifacts
 ├── scripts/                  # Helper build scripts (e.g. initramfs)
 └── manifest.toml             # Main system manifest
```

---

## 🧠 Design Principles

* **Immutable core** → The system is separated into 2 partitions: User Data and System Data
  * **System Data** contains the immutable system image that is swapped on every update
  * **User Data** contains the user's files and apps that were installed
* **Centralized configuration** → the **HyprRegistry** replaces scattered config files, allowing everything that would normally require a terminal to be configured from the settings app
* **UI-first system** → all core applications are built with HyprUI
* **Performance-first** → static Rust binaries, no unnecessary layers
* **Hot-reload everything** → configuration changes apply instantly; the compositor never restarts

---

## 🔧 Current Development State

> 🚧 **Status:** Active development (v0.1-dev)
> ✅ Kernel, initramfs, and SquashFS image boot successfully under QEMU
> 🧪 Work in progress: Init system

---

## 🪪 License

Hyprside is distributed under the **MIT License**.
