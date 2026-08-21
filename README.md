# 新型操作系统 / New Operating System

**非 Linux 衍生，完全重新设计的架构，以 Rust 为核心语言**

A new type of operating system not based on Linux, with a completely new architecture, using Rust as its core.

---

> 用新架构定义了一个新宇宙。  
> Define a universe with a new architecture.

---

## 新型操作系统宣言

# 序言：主权宣言——数字世界的第二次解放  
## Preamble: Sovereignty Declaration — The Second Liberation of the Digital World

我们，作为数字时代的公民，在此宣告：  
We, as citizens of the digital age, hereby declare:

旧世界的操作系统已经死去。  
The operating systems of the old world are dead.

它们不是我们的家园，而是我们的牢笼。  
They are not our homes; they are our cages.

它们不是我们的工具，而是我们的枷锁。  
They are not our tools; they are our chains.

它们不是我们的守护者，而是我们的监视者。  
They are not our guardians; they are our watchers.

---

### 一、旧世界的黑暗统治  
### I. The Dark Rule of the Old World

旧世界是资本主义的黑暗统治。少数巨头垄断了内核与标准，用封闭的协议和专利的壁垒，将数十亿用户变成了数字佃农。  
The old world is the dark rule of capitalism. A few giants monopolize kernels and standards, using closed protocols and patent barriers to turn billions of users into digital tenants.

他们用强制更新剥夺你的选择权。  
They strip you of choice with forced updates.

他们用遥测监控窥视你的一举一动。  
They spy on your every move with telemetry surveillance.

他们用“超级用户”的幽灵统治你的设备，让你在自己的电脑里，却是一个没有钥匙的房客。  
They rule your device with the ghost of the "superuser," making you a tenant without keys in your own computer.

传统内核是他们的私有财产，不是你的。  
The traditional kernel is their private property, not yours.

POSIX 是他们的法律，不是你的。  
POSIX is their law, not yours.

文件系统是他们的账本，每一笔数据都被记录、被分析、被收割。  
The filesystem is their ledger, where every piece of data is recorded, analyzed, and harvested.

系统调用是他们的收税站，每一次操作都要经过他们的关卡，留下你的足迹。  
System calls are their toll booths, where every operation passes through their checkpoints, leaving your footprints.

你无法主动选择内存模型，因为内存被他们分割、映射、隐藏。  
You cannot actively choose your memory model, because memory is partitioned, mapped, and hidden by them.

你无法在运行时改写系统权限，因为权限是他们赐予的，也可以随时收回。  
You cannot rewrite system permissions at runtime, because permissions are granted by them and can be revoked at any time.

你无法随意启用或关闭内核的特定功能模块，因为内核是他们的黑箱，你只能被动接受。  
You cannot freely enable or disable specific kernel modules, because the kernel is their black box, and you can only passively accept it.

这不是技术，这是统治。  
This is not technology; this is domination.

这不是进步，这是奴役。  
This is not progress; this is enslavement.

---

### 二、新世界的黎明  
### II. The Dawn of the New World

今天我们宣布：新世界已经到来。  
Today we declare: the new world has arrived.

新世界是共产主义的自由——代码开放，资源共有，每个用户都是自己设备的主人。  
The new world is the freedom of communism — open code, shared resources, every user the master of their own device.

在这里，没有超级用户，没有数字地主，没有后门，没有遥测。  
Here, there is no superuser, no digital landlord, no backdoor, no telemetry.

在这里，能力令牌取代权限位。每一个令牌都是平等的权利凭证，不可伪造，不可越界，只属于拥有者。  
Here, capability tokens replace permission bits. Every token is an equal credential of rights — unforgeable, non-transgressable, belonging only to its holder.

在这里，键值存储取代文件系统。数据不再被目录树和路径奴役，而是以最自然的方式存在、读取、修改。  
Here, key-value storage replaces filesystems. Data is no longer enslaved by directory trees and paths, but exists, reads, and changes in the most natural way.

在这里，语言安全取代硬件隔离。Rust 的编译期检查，让内存错误在代码运行之前就消亡，让越界访问成为不可能。  
Here, language safety replaces hardware isolation. Rust's compile-time checks kill memory errors before code runs, making out-of-bounds access impossible.

在这里，统一编址取代数据拷贝。所有资源映射到单一地址空间，零拷贝是常态，性能是架构内生的。  
Here, unified addressing replaces data copying. All resources are mapped into a single address space; zero-copy is the norm, and performance is inherent in the architecture.

我们是能力架构师，不再是系统的使用者。  
We are capability architects, no longer mere users of the system.

我们是数字世界的主人，不再是数字佃农。  
We are masters of the digital world, no longer digital tenants.

---

### 三、民用操作系统的承诺  
### III. The Promise of a Civilian Operating System

民用操作系统不是少数人的玩具，而是每个人的数字家园。  
A civilian operating system is not a toy for the few; it is the digital home for everyone.

它应该没有后门——因为用户是设备的唯一主人，任何未经用户知情的访问都是背叛。  
It should have no backdoors — because the user is the sole owner of the device; any access without the user's knowledge is betrayal.

它应该没有强制更新——因为用户的节奏由用户自己决定，系统永远服务用户，而不是绑架用户。  
It should have no forced updates — because the user's pace is decided by the user; the system serves the user, never kidnaps the user.

它应该没有遥测监控——因为用户的每一次点击、每一个字节都属于用户自己，不属于任何公司或政府。  
It should have no telemetry surveillance — because every click, every byte belongs to the user, not to any corporation or government.

它应该把主权还给用户，把安全交给语法，把性能交给架构。  
It should return sovereignty to the user, hand security to syntax, and give performance to architecture.

它不属于任何公司，不属于任何政府，只属于每一个使用它的人。  
It belongs to no corporation, no government — only to every person who uses it.

---

### 四、最终宣言  
### IV. The Final Declaration

我们相信，计算机可以重新开始。  
We believe that computing can start anew.

每一行代码都为现在而写，每一个令牌都是自由的凭证。  
Every line of code is written for the present; every token is a credential of freedom.

每一块内存都归用户所有，每一次调用都经用户授权。  
Every piece of memory belongs to the user; every call is authorized by the user.

我们不再继承历史的包袱，不再向旧世界的规则低头。  
We no longer inherit the baggage of history, no longer bow to the rules of the old world.

我们用新架构定义宇宙。  
We define a universe with a new architecture.

我们是新型冯·诺依曼内核。  
We are the New Von Neumann Kernel.

**让旧世界崩塌，让新世界诞生。**  
**Let the old world collapse, let the new world be born.**

**用户主权，代码自由，民用至上。**  
**User sovereignty, code freedom, civilian first.**

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

**本操作系统处于初级开发阶段 / This operating system is in the early stages of development.**

- **内核核心**（内存管理、页表、调度器、能力令牌引擎、中断处理）已稳定运行在 QEMU 和实机环境中。
- **多核支持**正在完善中。
- 完整的数学证明文档已覆盖所有核心子系统。

- **Kernel core** (memory management, page tables, scheduler, capability token engine, interrupt handling) is stable on QEMU and real hardware.
- **Multi‑core support** are under development.
- Complete mathematical proof documents cover all core subsystems.

---

## Github设计 / Github Design

- **main**分支在于此操作系统实现可装机后发布ISO镜像，不为贡献对象
- The **main** branch is about releasing an installable ISO image of this operating system, not intended for contributions
- **其他分支**可贡献， 因为目前处于开发操作系统中
- **Other branches** can contribute, because currently it's under operating system development

---

## 贡献指南 / Contribution Guidelines

### 内核 / The Kernel

**内核本身由项目创始人独立编写与维护，任何人可以提出建议，但建议由BDFL决定**

**The kernel itself is independently written and maintained by the project founder. Anyone can make suggestions, but the final call on suggestions is up to the BDFL.**

### 非内核功能 / Non‑Kernel Functionality

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
