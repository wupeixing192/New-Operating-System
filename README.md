# 新型操作系统 / New Operating System

**非 Linux 衍生，完全重新设计的架构，以 Rust 为核心语言**

A new type of operating system not based on Linux, with a completely new architecture, using Rust as its core.

---

## 核心设计 / Core Design

### 1. 彻底统一编址 / Thoroughly Unified Addressing

将所有资源（内存、I/O、文件、网络）映射到一个 64 位地址空间。消除了内核与用户空间、内存与存储之间无数次的数据复制。

Map all resources (memory, I/O, files, network) to a single 64‑bit address space. This eliminates countless data copies between kernel and user space, memory and storage.

### 2. 语言即是隔离墙 / Language as the Isolation Wall

用 Rust 的编译期检查（类型系统、所有权、借用检查）替代 MMU 硬件隔离。实现极其轻量的上下文切换和划时代的安全保障。

Use Rust's compile‑time checks (type system, ownership, borrowing) instead of MMU hardware isolation. This means extremely lightweight context switching and groundbreaking security.

### 3. 能力授权引擎 / Capability Authorization Engine

不可伪造的令牌作为一切资源访问的唯一凭证。程序不靠“拥有更多权限”，而是靠“能证明自己应该被允许”。

Non‑forged tokens serve as the sole credential for resource access. A program is not judged by "who has more permissions" but by "who can prove that it should be allowed."

---

## 技术参考 / Technical References

| 理念 / Concept | 参考项目 / Reference Projects |
| :--- | :--- |
| 彻底统一编址 / Unified Addressing | Unikernel (OSv, MielinOS, μFork) |
| 能力授权引擎 / Capability Engine | RedLeaf OS, Tock OS, ATLAS |
| 语言即是隔离墙 / Language as Isolation | RedLeaf OS, Tock OS, herkos, Asterinas |

---

## 技术的“合金”与“提纯” / Technology “Alloy” and “Purification”

本框架的真正独创性在于：将上述三种理念以“Rust 语言安全”为核心逻辑进行“合金式”融合，让它们互为因果，形成一个逻辑严密、相互增强的有机整体。如果说 RedLeaf 等项目是“用工具改造现有的房子”，那这个新型操作系统就如同“从零开始，重新发明一种更坚固、更轻盈的建材来造房子”。

同时，该框架实现了对思想实验的哲学提纯。例如，Unikernel 通常在虚拟机之上运行，隔离依赖于 Hypervisor。而这个操作系统直接在裸机上运行，完全依赖 Rust 编译器保证内部组件的隔离，打造了一个性能损耗接近于零的“纯软件虚拟机”环境。

The true originality of this framework lies in the clever “alloy‑style” fusion of the three concepts with the core logic of “Rust language security”. They become mutually causal, forming a logically rigorous and mutually reinforcing organic whole. If projects like RedLeaf are about “transforming existing houses with tools”, then this new operating system is like “reinventing a stronger and lighter building material from scratch to build houses”.

At the same time, the framework achieves philosophical purification of thought experiments. For example, Unikernel typically runs on top of a hypervisor, relying on it for isolation. This operating system runs directly on bare metal and relies entirely on the Rust compiler to guarantee isolation of its internal components, creating a “pure‑software virtual machine” environment with near‑zero performance loss.

---

## 项目状态 / Project Status

**⚠️ 本操作系统处于早期开发阶段 / ⚠️ This operating system is in an early development stage.**

- **内核核心**（内存管理、页表、调度器、能力令牌引擎、中断处理）已稳定运行在 QEMU 和实机环境中。
- **多核支持**与**用户态切换**正在完善中。
- 完整的数学证明文档已覆盖所有核心子系统。

- **Kernel core** (memory management, page tables, scheduler, capability token engine, interrupt handling) is stable on QEMU and real hardware.
- **Multi‑core support** and **user‑mode switching** are under development.
- Complete mathematical proof documents cover all core subsystems.

---

## 贡献指南 / Contribution Guidelines

### 🔒 内核 / The Kernel

**内核本身由项目创始人独立编写与维护，不接受外部贡献。**

The kernel itself is written and maintained exclusively by the project founder. **External contributions to the kernel are not accepted.**

### 🧩 非内核功能 / Non‑Kernel Functionality

**任何人都可以为该操作系统编写非内核功能**，例如：

Anyone is welcome to contribute **non‑kernel functionality**, for example:

- 兼容驱动 / Driver compatibility
- 网络栈 / Network stack
- 键‑值对象存储服务 / Key‑value object storage services
- 发布‑订阅通道模块 / Publish‑subscribe channel modules
- 用户态应用 / User‑space applications
- 图形服务 / Graphics services
- WASI 兼容层 / WASI compatibility layer

这些贡献请通过 **分支（branch）** 提交，主分支仅保留内核核心代码。

Please submit these contributions via **branches**. The main branch only contains the kernel core code.

---

## 宣言 / Declaration

- 内核必须原生支持基于 VT‑d 的 GPU 直通能力，向用户态提供 GPU 能力令牌，而非显卡抽象层。图形性能不得低于裸机水平的 95%。

- 本系统永不引入 POSIX 语义。任何绕过能力令牌直接操作物理地址的行为，均视为对系统安全模型的根本破坏。

- 系统启动时间（从按下电源到可交互）不超过 3 秒。应用安装不得修改全局状态，通过键值对象存储实现独立沙箱。系统更新采用原子替换，永不强制重启。

---

- The kernel must natively support VT‑d based GPU pass‑through, providing GPU capability tokens to user mode rather than a graphics card abstraction layer. Graphics performance must not be lower than 95% of bare metal level.

- This system will never introduce POSIX semantics. Any behavior that bypasses capability tokens and directly operates on physical addresses is considered a fundamental breach of the system security model.

- System startup time (from power‑on to interactive) shall not exceed 3 seconds. Application installation must not modify global state; independent sandboxing is achieved through key‑value object storage. System updates use atomic replacement and never force a restart.

---

## 链接 / Links

- [GitHub 仓库 / Repository](https://github.com/wupeixing192/New-Operating-System)
- [Gitee 镜像 / Mirror](https://gitee.com/wupeixing192/New-Operating-System)
- [技术宪法 / Technical Constitution](docs/宪法.md)
- [开发文档 / Development Docs](docs/)
- [数学证明 / Mathematical Proofs](docs/proofs/)
