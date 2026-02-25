# Ardos OS

**Ardos OS** is a next-generation immutable Linux operating system built around modern UI principles and GPU acceleration.
It combines native performance, deep system integration, and full user customization — without sacrificing determinism or security.

---

## 🧭 Philosophy

Ardos OS is a love letter to the Linux kernel — a demonstration of what it can truly become.

- Takes inspiration from mobile operating systems like Android and iOS, and from Apple’s UX/UI philosophy
- The base system is **immutable**, delivered as a **SquashFS image**
- User configuration is stored in the **ArdosRegistry**, not traditional dotfiles
- The entire UI stack is powered by **Ardos UI**, a GPU-native Rust framework
- The compositor is a fork of **Hyprland**, adapted for non-technical users

---

## 🧩 (Planned) Core Components

| Component | Description |
|------------|-------------|
| **Ardos UI** | Declarative immediate-mode UI framework (Rust + Skia + Clay) |
| **Ardos DE** | Complete desktop environment built with Ardos UI |
| **Ardos Theme** | System-wide theming and visual identity manager that works consistently across toolkits and apps |
| **Shift** | A replacement for the Linux TTY system, enabling smooth transitions between screens |
| **Ardos Init** | Minimal and purpose-built init system designed specifically for Ardos OS |
| **Ardos Packer** | Unified build system for the kernel, image, and initrd |
| **Kernel Bombproof** | Hardened Linux kernel fork with patches to prevent UI freezes under heavy workloads |

---

## ⚙️ Build Pipeline Overview

Ardos OS is fully built via **Ardos Packer**:

1. `manifest.toml` defines the system components
2. Ardos Packer compiles the **kernel**, **initrd**, and all **packages**
3. The result is an immutable SquashFS system image
4. You can test it directly in QEMU with UEFI boot

```bash
ardos-packer vm run
```

This command performs a full build and automatically boots the OS.

---

## 🧱 Repository Structure

```
ardos/
 ├── packages/
 │   ├── ArdosPacker/               # Build tool
 │   ├── ArdosInit/                 # Stage-1 init
 │   ├── Ardos DE/               # Desktop environment
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
* **Centralized configuration** → the **registry** replaces scattered config files, allowing everything that would normally require a terminal to be configured from the settings app
* **UI-first system** → all core applications are built with Ardos UI
* **Performance-first** → static Rust binaries, no unnecessary layers
* **Hot-reload everything** → configuration changes apply instantly; the compositor never restarts

---

## 🔧 Current Development State

> 🚧 **Status:** Active development (v0.1-dev)
> ✅ Kernel, initramfs, and SquashFS image boot successfully under QEMU
> 🧪 Work in progress: Init system

---

## 🪪 License

Ardos OS is distributed under the **MIT License**.
