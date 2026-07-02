<<<<<<< HEAD
# bootloader

[![Docs](https://docs.rs/bootloader/badge.svg)](https://docs.rs/bootloader)
[![Build Status](https://github.com/rust-osdev/bootloader/actions/workflows/build.yml/badge.svg)](https://github.com/rust-osdev/bootloader/actions/workflows/build.yml)
[![Join the chat at https://rust-osdev.zulipchat.com](https://img.shields.io/badge/zulip-join_chat-brightgreen.svg)](https://rust-osdev.zulipchat.com)

An experimental x86_64 bootloader that works on both BIOS and UEFI systems. Written in Rust and some inline assembly, buildable on all platforms without additional build-time dependencies (just some `rustup` components).

## Requirements

You need a nightly [Rust](https://www.rust-lang.org) compiler with the `llvm-tools-preview` component, which can be installed through `rustup component add llvm-tools-preview`.

## Usage

To use this crate, you need to adjust your kernel to be bootable first. Then you can create a bootable disk image from your compiled kernel. These steps are explained in detail below.

### Migrating from older bootloader version

If you're already using an older version of the `bootloader` crate, follow our [migration guides](docs/migration).

### Starting from scratch

Our [basic example](examples/basic/basic-os.md) showcases an OS that boots a minimal kernel using `bootloader`.

### Using an existing kernel

To combine your kernel with `bootloader` and create a bootable disk image, follow these steps:

#### Make your kernel compatible with `bootloader`

- Add a dependency on the `bootloader_api` crate in your kernel's `Cargo.toml`.
- Your kernel binary should be `#![no_std]` and `#![no_main]`.
- Define an entry point function with the signature `fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> !`. The function name can be arbitrary.
  - The `boot_info` argument provides information about available memory, the framebuffer, and more. See the API docs for `bootloader_api` crate for details.
- Use the `entry_point` macro to register the entry point function: `bootloader_api::entry_point!(kernel_main);`
  - The macro checks the signature of your entry point function and generates a `_start` entry point symbol for it. (If you use a linker script, make sure that you don't change the entry point name to something else.)
  - To use non-standard configuration, you can pass a second argument of type `&'static bootloader_api::BootloaderConfig` to the `entry_point` macro. For example, you can require a specific stack size for your kernel:
    ```rust
    const CONFIG: bootloader_api::BootloaderConfig = {
        let mut config = bootloader_api::BootloaderConfig::new_default();
        config.kernel_stack_size = 100 * 1024; // 100 KiB
        config
    };
    bootloader_api::entry_point!(kernel_main, config = &CONFIG);
    ```
- Compile your kernel to an ELF executable by running **`cargo build --target x86_64-unknown-none`**. You might need to run `rustup target add x86_64-unknown-none` for BIOS and `rustup target add x86_64-unknown-uefi` for UEFI before to download precompiled versions of the `std`, `core` and `alloc` crates.
- Thanks to the `entry_point` macro, the compiled executable contains a special section with metadata and the serialized config, which will enable the `bootloader` crate to load it.

#### Creating a bootable image

- Move your full kernel code into a `kernel` subdirectory.
- Create a new `os` crate at the top level
    ```sh
    $ cargo init --bin
    ```
- Define a [workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html).
    ```toml
    # in Cargo.toml
    [workspace]
    resolver = "3"
    members = []
    ```
- Add your kernel as a workspace member.
    ```sh
    $ cargo new kernel --bin
    ```
- Enable the workspace to build your kernel:
  - Set up an [artifact dependency](https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#artifact-dependencies) to add your `kernel` crate as a `build-dependency`:
      ```toml
      # in Cargo.toml
      [build-dependencies]
      kernel = { path = "kernel", artifact = "bin", target = "x86_64-unknown-none" }
      ```
      Enable the unstable artifact-dependencies feature:
      ```toml
      # .cargo/config.toml
      [unstable]
      bindeps = true
      ```
      Experimental features are only available on the nightly channel:
      ```toml
      # rust-toolchain.toml
      [toolchain]
      channel = "nightly"
      targets = ["x86_64-unknown-none", "x86_64-unknown-uefi"]
      ```
  - Alternatively, you can use [`std::process::Command`](https://doc.rust-lang.org/stable/std/process/struct.Command.html) to invoke the build command of your kernel in the `build.rs` script.
- Create a [`build.rs`](https://doc.rust-lang.org/cargo/reference/build-scripts.html) build script in the `os` crate. See our [disk image creation template](docs/create-disk-image.md) for a more detailed example.
  - Obtain the path to the kernel executable. When using an artifact dependency, you can retrieve this path using `std::env::var_os("CARGO_BIN_FILE_MY_KERNEL_my-kernel")`
  - Use `bootloader::UefiBoot` and/or `bootloader::BiosBoot` to create a bootable disk image with your kernel.
- Do something with the bootable disk images in your `main.rs` function. For example, run them with QEMU.

See our [disk image creation template](docs/create-disk-image.md) for a more detailed example.

## Architecture

This project is split into three separate entities:

- A [`bootloader_api`](./api) library with the entry point, configuration, and boot info definitions.
  - Kernels should include this library as a normal cargo dependency.
  - The provided `entry_point` macro will encode the configuration settings into a separate ELF section of the compiled kernel executable.
- [BIOS](./bios) and [UEFI](./uefi) binaries that contain the actual bootloader implementation.
  - The implementations share a higher-level [common library](./common).
  - Both implementations load the kernel at runtime from a FAT partition. This FAT partition is created
  - The configuration is read from a special section of the kernel's ELF file, which is created by the `entry_point` macro of the `bootloader_api` library.
- A `bootloader` library to create bootable disk images that run a given kernel. This library is the top-level crate in this project.
  - The library builds the BIOS and UEFI implementations in the [`build.rs`](./build.rs).
  - It provides functions to create FAT-formatted bootable disk images, based on the compiled BIOS and UEFI bootloaders.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
=======
# 新型操作系统/New-Operating-System
A new type of operating system not based on Linux, with a completely new architecture, using Rust as its core.

1. 彻底统一编址：将所有资源映射到一个64位地址空间。这消除了内核与用户空间、内存与存储之间无数次的数据复制。

2. 语言即是隔离墙：用Rust的编译期检查，替代MMU硬件隔离。这意味着极其轻量的上下文切换，和实现上的划时代安全。

3. 能力授权引擎：不具伪造性的令牌作为一切资源访问的唯一凭证。这不只是安全，它重新定义了“程序”与“权限”的关系——不是谁拥有更多权限，而是谁能证明它应该被允许。


1. Thoroughly unify addressing: map all resources to a 64 bit address space. This eliminates countless instances of data replication between the kernel and user space, memory and storage.

2. Language is like a firewall: using Rust's compile time check instead of MMU hardware isolation. This means extremely lightweight context switching and groundbreaking security in implementation.

3. Capability authorization engine: Non forged tokens serve as the sole credential for accessing all resources. This is not just about security, it redefines the relationship between "programs" and "permissions" - not who has more permissions, but who can prove that it should be allowed.

彻底统一编址：将所有资源（内存、I/O、文件、网络）映射到单一64位虚拟地址空间。
例如：Unikernel（如OSv、MielinOS）、μFork等。
能力授权引擎：使用能力作为资源访问的唯一凭证，由内核统一创建和验证。
例如：RedLeaf OS（使用Rust语言特性实现隔离）、Tock OS、ATLAS。
语言即是隔离墙：用Rust的类型系统、所有权和借用检查，替代传统MMU的硬件隔离。 
例如：RedLeaf OS、Tock OS、herkos、Asterinas星绽。

Thoroughly unify addressing: map all resources (memory, I/O, files, network) to a single 64 bit virtual address space.
For example: Unikernel (such as OSv, MielinOS), μ Fork, etc.
Capability Authorization Engine: Using capabilities as the sole credential for resource access, created and validated by the kernel.
For example: RedLeaf OS (implementing isolation using Rust language features) Tock OS、ATLAS。
Language is like a barrier: using Rust's type system, ownership, and borrowing checks to replace traditional MMU hardware isolation.  
For example: RedLeaf OS, Tock OS, herkos, Asterinas Starburst.

技术的“合金”与“提纯”这个新型操作系统的框架真正独创性在于：将上述三种理念以“Rust语言安全”这一核心逻辑进行了一次巧妙的“合金式”融合。让它们互为因果，形成了一个逻辑严密、相互增强的有机整体。如果说RedLeaf等项目是“用工具改造现有的房子”，那这个新型操作系统就如同“从零开始，重新发明一种更坚固、更轻盈的建材来造房子”。同时，该框架实现了对思想实验的哲学提纯。例如，Unikernel通常在虚拟机之上运行，隔离依赖于Hypervisor。而这个操作系统的目标是直接在裸机上运行，并且完全依赖Rust编译器来保证其内部组件的隔离，这就好比打造了一个性能损耗接近于零的“纯软件虚拟机”环境。

The true originality of the framework for this new operating system, which combines "alloy" and "purification" of technology, lies in the clever "alloy style" fusion of the three concepts with the core logic of "Rust language security". Let them be mutually causal, forming a logically rigorous and mutually reinforcing organic whole. If projects like RedLeaf are about "transforming existing houses with tools," then this new operating system is like "reinventing a stronger and lighter building material from scratch to build houses. At the same time, the framework achieves philosophical purification of thought experiments. For example, Unikernel typically runs on top of a virtual machine and relies on the hypervisor for isolation. The goal of this operating system is to run directly on bare metal and rely entirely on the Rust compiler to ensure the isolation of its internal components, which is like creating a "pure software virtual machine" environment with almost zero performance loss.

## 注意 / Notice

**本操作系统处于早期开发阶段。**

任何人都可以为该操作系统编写**非内核功能**（例如兼容驱动、网络栈、键‑值对象存储服务、发布‑订阅通道模块、用户态应用等）。**内核本身由项目创始人独立编写与维护，不接受外部贡献。**

**This operating system is in an early development stage.**

Anyone is welcome to contribute **non-kernel functionality** (e.g., driver compatibility, network stack, key‑value object storage services, pub‑sub channel modules, user‑space applications, etc.). **The kernel itself is written and maintained exclusively by the project founder. External contributions to the kernel are not accepted.**

# 宣言/Declaration:

内核必须原生支持基于VT-d的GPU直通能力，向用户态提供GPU能力令牌，而非显卡抽象层。图形性能不得低于裸机水平的95%。

本系统永不引入POSIX语义。任何绕过能力令牌直接操作物理地址的行为，均视为对系统安全模型的根本破坏。

系统启动时间（从按下电源到可交互）不超过3秒。应用安装不得修改全局状态，通过键值对象存储实现独立沙箱。系统更新采用原子替换，永不强制重启。

The kernel must natively support GPU pass through capabilities based on VT-d, providing GPU capability tokens to user mode rather than the graphics card abstraction layer. The graphics performance shall not be lower than 95% of the bare metal level.

This system will never introduce POSIX semantics. Any behavior that bypasses the ability token and directly operates on physical addresses is considered a fundamental breach of the system security model.

The system startup time (from pressing the power button to being interactive) shall not exceed 3 seconds. Application installation cannot modify the global state, and an independent sandbox is implemented through key value object storage. The system update uses atomic substitution and never forces a restart.
>>>>>>> 27d372b613fa270f660ae95298d578a03720215f
