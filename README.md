# 新型操作系统 / New Operating System
# 新型冯·诺依曼内核  
New Von Neumann Kernel

> 用新架构定义了一个新宇宙。  
> Define a universe with a new architecture.

---

### 当前内核版本：v1.0

---
## 新型内核宣言

# 序言：主权宣言——计算不再被过去定义
# Preamble: Sovereignty Declaration — Computing No Longer Defined by the Past

**我们，作为数字时代的公民，在此宣告：**
**We, as citizens of the digital age, hereby declare:**

**传统操作系统不是我们的宿命。**
**Legacy operating systems are not our destiny.**

**POSIX 不是我们的语言。**
**POSIX is not our language.**

**超级用户不是我们的统治者。**
**The superuser is not our ruler.**

**文件系统不是我们唯一的记忆方式。**
**The filesystem is not our only way of remembering.**

**系统调用不是我们唯一的沟通途径。**
**The system call is not our only means of communication.**

**我们曾被迫接受一套陈旧的架构：它诞生于分时共享的年代，带着目录树、权限位、进程隔离的历史包袱。这套架构曾经辉煌，但它也把我们变成了被动的“用户”——我们无法真正掌控自己的设备，无法自由选择资源的组织方式，无法摆脱内核的隐形统治。我们无法主动选择内存模型，无法在运行时改写系统权限，无法随意启用或关闭内核的特定功能模块。**
**We were once forced to accept an outdated architecture: born in the era of time-sharing, burdened with directory trees, permission bits, and process isolation. That architecture once shined, but it also turned us into passive "users" — unable to truly control our own devices, unable to freely choose how resources are organized, unable to escape the invisible rule of the kernel. We could not actively choose our memory model, could not rewrite system permissions at runtime, could not freely enable or disable specific kernel modules.**

**旧世界是资本主义的黑暗统治——少数巨头垄断内核与标准，用封闭协议和专利壁垒榨取用户，用强制更新和遥测监控将用户变成数字佃农。新世界是共产主义的自由——代码开放，资源共有，每个用户都是自己设备的主人，每个令牌都是平等的权利凭证。没有超级用户，没有数字地主，没有后门，只有属于人民的主权。**
**The old world is the dark rule of capitalism — a few giants monopolize kernels and standards, exploiting users with closed protocols and patent barriers, turning users into digital tenants through forced updates and telemetry surveillance. The new world is the freedom of communism — open code, shared resources, every user the master of their own device, every token an equal credential of rights. No superuser, no digital landlords, no backdoors, only sovereignty belonging to the people.**

**今天我们宣布：我们不再做系统的“使用者”，我们要做系统的“主人”。我们要用能力令牌取代权限位，用键值存储取代文件系统，用语言安全取代硬件隔离，用统一编址取代数据拷贝。我们不再是“能力架构师”的旁观者，我们就是能力架构师本身。**
**Today we declare: we will no longer be mere "users" of the system; we will be the masters of the system. We will replace permission bits with capability tokens, replace filesystems with key-value stores, replace hardware isolation with language safety, replace data copying with unified addressing. We are no longer spectators of the "capability architect" — we are the capability architects ourselves.**

**民用操作系统不是少数人的玩具，而是每个人的数字家园。它应该没有后门，没有强制更新，没有遥测监控。它应该把主权还给用户，把安全交给语法，把性能交给架构。它不属于任何公司，不属于任何政府，只属于每一个使用它的人。**
**A civilian operating system is not a toy for the few; it is the digital home for everyone. It should have no backdoors, no forced updates, no telemetry surveillance. It should return sovereignty to the user, hand security to syntax, and give performance to architecture. It belongs to no corporation, no government — only to every person who uses it.**

**我们相信，计算机的操作系统可以重新开始。每一行代码都为现在而写，每一个令牌都是自由的凭证。**
**We believe that computing can start anew. Every line of code is written for the present; every token is a credential of freedom.**

# 用新架构定义宇宙。
# Define a universe with a new architecture.

---

## 内核介绍  
## Kernel Introduction

**新型冯·诺依曼内核** 是一个从零开始设计和实现的全新操作系统内核，不基于 Linux 或其他现有内核框架。  
**New Von Neumann Kernel** is an operating system kernel designed and implemented from scratch, not based on Linux or any existing kernel framework.

内核围绕三大哲学支柱构建：  
The kernel is built around three philosophical pillars:

1. **彻底统一编址**  
   **Unified Addressing**  
   所有资源（内存、I/O、文件、网络）映射到单一 64 位虚拟地址空间，消除内核与用户态之间的数据拷贝，实现零拷贝通信。  
   All resources (memory, I/O, files, network) are mapped into a single 64-bit virtual address space, eliminating data copies between kernel and user space for zero-copy communication.

2. **语言即是隔离墙**  
   **Language as Isolation Wall**  
   用 Rust 的类型系统、所有权和借用检查，替代传统 MMU 硬件隔离。编译期消灭内存漏洞，实现单地址空间内的安全。Intel MPK 作为硬件兜底，防止不安全 C 代码越界。  
   Rust's type system, ownership, and borrow checking replace traditional MMU hardware isolation, eliminating memory vulnerabilities at compile time. Intel MPK serves as a hardware fallback to prevent unsafe C code from crossing boundaries.

3. **能力授权引擎**  
   **Capability Authorization Engine**  
   不可伪造的能力令牌（Capability Token）是访问任何资源的唯一凭证。无超级用户（root），无后门。令牌包含 `auth_hash` 防伪字段，使用数字签名机制防止伪造。每个令牌指向对象表中的一个条目，明确限制了可访问的内存范围（base + limit）。  
   An unforgeable Capability Token is the sole credential for accessing any resource. No root, no backdoors. Tokens contain an `auth_hash` anti-forgery field and use digital signatures to prevent forgery. Each token points to an ObjectTable entry, explicitly limiting accessible memory range (base + limit).

内核极简：只做创建、验证、撤销能力令牌。所有驱动、文件系统、网络栈均在 Ring 3 用户态运行。无 POSIX，无系统调用，原子更新。  
The kernel is minimal: it only creates, verifies, and revokes capability tokens. All drivers, filesystems, and network stacks run in Ring 3 user mode. No POSIX, no system calls, atomic updates.

---

## 当前状态  
## Current Status

**早期开发阶段**。内核核心模块正在逐步实现，尚未达到可日常使用状态。  
**Early development stage**. Core kernel modules are being implemented progressively; not yet ready for daily use.

**已完成：**  
**Completed:**

- 16 位实模式汇编引导扇区（512 字节），完成“原子回路验证”  
- 16-bit real-mode assembly boot sector (512 bytes), completed "Atomic Loop Verification"
- 64 位 UEFI Rust 内核，在 QEMU 中运行  
- 64-bit UEFI Rust kernel running in QEMU
- 动态能力令牌系统（创建、验证、委托、撤销）  
- Dynamic capability token system (create, verify, delegate, revoke)
- 物理内存分配器（动态位图）  
- Physical memory allocator (dynamic bitmap)
- 对象表（ObjectTable），令牌精确锁定资源边界  
- ObjectTable, tokens precisely lock resource boundaries
- 协作式任务调度器（yield_now、上下文切换）  
- Cooperative task scheduler (yield_now, context switching)
- 安全入口桩（IDT 中断处理，GPF/Page Fault/Double Fault 接入令牌裁决）  
- Secure entry stub (IDT interrupt handling, GPF/Page Fault/Double Fault routed to token adjudication)
- 帧缓冲图形输出（8×8 字体，0-9 和 A-Z）  
- Framebuffer graphics output (8×8 font, 0-9 and A-Z)
- 完整的密码学体系设计（根密钥、发布密钥、用户密钥分级）  
- Complete cryptographic hierarchy design (root key, release key, user key)

**下一步计划：**  
**Next steps:**

- Ring 3 用户态切换  
- Ring 3 user-mode transition
- WASI 兼容层  
- WASI compatibility layer
- 键值存储服务（用户态）  
- Key-value store service (user space)
- 桌面
- Desktop
- 硬件代码
- hardware code

---

## 密钥状态  
## Key Status

项目采用分级密钥管理：  
The project uses hierarchical key management:

- **根密钥**：离线保存，绝不入库。根公钥固化在 ROM 中，作为信任链绝对起点。  
- **Root key**: Stored offline, never committed to repository. Root public key is fused in ROM as the absolute starting point of trust.
- **发布密钥**：用于签名内核和系统更新。当前发布公钥位于 `key/release-pubkey.der`，证书 `release.crt` 由根私钥签名，用于验证发布公钥合法性。  
- **Release key**: Used to sign the kernel and system updates. The release public key is located at `key/release-pubkey.der`; certificate `release.crt` is signed by the root private key to validate the release public key.
- **用户密钥**：待实现，将用于用户身份认证和数据加密。  
- **User keys**: To be implemented, will be used for user authentication and data encryption.

**注意**：仓库中不包含任何私钥。所有私钥必须由项目所有者离线保管。  
**Note**: No private keys are included in the repository. All private keys must be kept offline by the project owner.

---

## 如何运行  
## How to Run

### 环境要求  
### Requirements

- Ubuntu 24.04（或其他 Linux 发行版）  
- Ubuntu 24.04 (or other Linux distribution)
- Rust 工具链（nightly 版本，因使用 `#![no_std]` 和 `bootloader_api`）  
- Rust toolchain (nightly, due to `#![no_std]` and `bootloader_api`)
- gcc工具
- gcc Tool
- QEMU 系统模拟器  
- QEMU system emulator
- OVMF UEFI 固件  
- OVMF UEFI firmware
- `cargo` 和 `rust-src` 组件  
- `cargo` and `rust-src` components

### 构建与运行  
### Build and Run

# 安装 Rust nightly 和 `rust-src`：  
   Install Rust nightly and `rust-src`:
   ```bash
   rustup default nightly
   rustup component add rust-src
   ```

# 安装 QEMU 和 OVMF：  
   Install QEMU and OVMF:
   ```bash
   sudo apt update
   sudo apt install qemu-system-x86 ovmf
   ```

# 构建内核：  
   Build the kernel:
   ```bash
   cargo build --target x86_64-unknown-none
   ```

# 使用 QEMU 运行：  
   Run with QEMU :
   ```bash
   mkdir -p esp/EFI/BOOT
   cp target/x86_64-unknown-uefi/debug/my_uefi_kernel.efi esp/EFI/BOOT/BOOTX64.EFI
   qemu-system-x86_64 -bios /usr/share/ovmf/OVMF.fd -drive file=fat:rw:esp/,format=raw -serial stdio -no-reboot -no-shutdown
   ```
 **或/or**
   ```bash
   qemu-system-x86_64 -bios /usr/share/ovmf/OVMF.fd -kernel target/x86_64-unknown-uefi/debug/my_uefi_kernel.efi -serial stdio -no-reboot -no-shutdown
   ```

---

## 注意事项
## Notes

# 本新型内核的已知问题 / Known issues of this new kernel

- **任务池数量上限**：在src/main.rs内核文件中，第57行：/ **Task pool limit**: In the core file src/main.rs, line 57:
   ```rust
   pub const MAX_TASKS_PER_CORE: usize = 10;  // 每个 CPU 核心最多可容纳的任务（执行上下文）数量
   ```
   BDFL在编写与调试这个内核中，**发现调到64,甚至32在终端都没有输出，很有可能的原因是栈溢出**
   While writing and debugging this kernel, the BDFL **found that adjusting it to 64, or even 32, produced no output in the terminal, and the most likely reason is a stack overflow**.

  在：系统4，时间分配调度器这一部分中，位于第493行：/ In: System 4, the time-sharing scheduler section, at line 493:
  
  ```rust
  const STACK_SIZE: usize = 4096 * 2; // 每个任务的栈大小（4 KiB）
  ```
  
  BDFL不断修改此值，**发现如果数字过大，可能出现以下情况**：/ BDFL keeps changing this value and **found that if the number is too large, the following might happen**:
  
  ```bash
  BdsDxe: failed to load Boot0001 "UEFI Non-Block Boot Device" from VenMedia(1428F772-B64A-441E-B8C3-9EBDD7F893C7): Not Found

  >>Start PXE over IPv4.
  ```
  
  **原因：/ Reason:**
  
  **为了减小 PE 文件的静态数据，让 OVMF 能加载。**/**To reduce the static data of the PE file so that OVMF can load it.**
  
  BDFL最初的配置：/ The original setup of the BDFL:
  
  PerCoreData 里有 task_pool: [Option<Task>; MAX_TASKS_PER_CORE]，而 Task 里有 stack: [u8; STACK_SIZE]。
  In PerCoreData, there's task_pool: [Option<Task>; MAX_TASKS_PER_CORE], and Task has stack: [u8; STACK_SIZE].

  静态数据计算：/ Static data calculation:
  
  · 每个任务栈 = 16KB / Each task stack = 16KB
  
  · 每核心 64 个任务 = 64 × 16KB = 1MB / 64 tasks per core = 64 × 16KB = 1MB
  
  · 256 核心 = 256 × 1MB = 256MB / 256 cores = 256 × 1MB = 256MB
  
  · 加上 AP_STACKS、BITMAP、HEAP 等，总计 > 260MB / Including AP_STACKS, BITMAP, HEAP, etc., the total is over 260MB
  
  **OVMF 加载 PE 文件时，必须预留整个 SizeOfImage 的虚拟地址空间。260MB 超过了固件启动阶段的可用内存，导致 Out of Resources。**
  **When OVMF loads a PE file, it must reserve the entire virtual address space of SizeOfImage. 260MB exceeds the available memory during the firmware boot stage, causing an Out of Resources error.**
  
  **把 STACK_SIZE 从 16KB 改成 8KB，每核心任务池从 1MB 降到 512KB，总静态数据大幅减小，PE 文件才能被 OVMF 加载。**
  **Change STACK_SIZE from 16KB to 8KB, reduce each core task pool from 1MB to 512KB, and the total static data will shrink a lot so that the PE file can be loaded by OVMF.**

- **关于密钥 / About the key**

  **BDFL决定：当键值存储开发为可用时，将根密钥描定于内核，与键值存储形成一个完整的信任链 / BDFL decided: when the key-value store becomes available, the root key will be hard-coded into the kernel, forming a complete chain of trust with the key-value store.**

---

#### 由于项目处于早期开发阶段，**所以我们的文档是最终目标，不是直接实现**：
#### Since the project is in the early stages of development, **our documentation is the end goal, not the direct implementation**:

>**BDFL声明：/BDFL statement:**
>**能跑 > 能用 > 安全**
>**Can run > Can use > Safe**

---

## 法律声明  
## Legal Notice

本项目采用 **GPL 3.0** 许可证。详细信息请参阅 [LICENSE](LICENSE) 文件。  
This project is licensed under **GPL 3.0**. See the [LICENSE](LICENSE) file for details.

项目名称“新型冯·诺依曼内核”及相关标识为作者所有。未经授权，不得用于商业用途或冒名发布。  
The project name "New Von Neumann Kernel" and related logos are owned by the author. Unauthorized commercial use or impersonation is prohibited.

本软件按“原样”提供，不提供任何明示或暗示的担保。作者不对使用本软件造成的任何损失负责。  
This software is provided "as is", without warranty of any kind. The author is not liable for any damages arising from its use.

---

## 联系方式  
## Contact

- 作者 / Author：wupeixing
- 邮箱 / Email : 367125693@qq.com

---

## 核心术语  
## Core Terminology

- **原子回路**：为验证能力令牌模型设计的最小裸机实验，在 512 字节实模式引导扇区中动态验证了令牌的创建、合法通过、越权拒绝和伪造拦截。  
- **Atomic Loop**: A minimal bare-metal experiment to verify the capability token model, dynamically validating token creation, legal passage, unauthorized rejection, and forgery interception in a 512-byte real-mode boot sector.
- **能力令牌**：不可伪造的数字凭证，包含 `auth_hash` 防伪字段，指向对象表中的具体资源条目，是系统中唯一的资源访问凭证。  
- **Capability Token**: An unforgeable digital credential containing an `auth_hash` anti-forgery field, pointing to a specific entry in the ObjectTable; it is the sole resource access credential.
- **对象表**：内核维护的全局表，每个条目记录一段受保护内存的基址、长度和权限。  
- **ObjectTable**: A global table maintained by the kernel; each entry records the base, length, and permissions of a protected memory region.
- **语言即隔离墙**：用 Rust 编译期安全检查替代 MMU 硬件隔离，实现单地址空间内的安全。  
- **Language as Isolation Wall**: Using Rust's compile-time safety checks to replace MMU hardware isolation for safe single-address-space operation.
- **安全入口桩**：所有中断和异常的唯一入口，负责验证令牌、冻结违规上下文、记录故障签名。  
- **Secure Entry Stub**: The sole entry point for all interrupts and exceptions, responsible for verifying tokens, freezing offending contexts, and recording fault signatures.
- **BDFL**：终身仁慈独裁者，项目的唯一最高决策者。  
- **BDFL**: Benevolent Dictator for Life, the sole highest decision-maker of the project.
- **固化根因**：系统 ROM 中固化的最小信任根，负责验证内核签名，是所有信任链的绝对起点。  
- **Immutable Root of Trust**: The minimal trust root fused in system ROM, responsible for verifying the kernel signature; it is the absolute starting point of the trust chain.
- **MPK 影子墙**：用 Intel MPK 硬件特性为不安全 C 代码提供硬件级隔离兜底，防止内存越界。  
- **MPK Shadow Wall**: Using Intel MPK hardware features to provide hardware-level isolation fallback for unsafe C code, preventing memory overruns.

## 2026新型操作系统开发团队 / New Operating System Development Team

## 为民用而生 / Born for the people
