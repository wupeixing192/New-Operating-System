# 新型操作系统 · 设计哲学  
# New Operating System · Design Philosophy

> **从零构建，为民用而生。**  
> **Built from scratch, for the people.**

---

## 一、我们为什么重新发明操作系统？  
## I. Why Are We Reinventing the Operating System?

现有的操作系统，无论是闭源的 Windows 还是开源的 Linux，都背负着 40 年以上的历史包袱：

Existing operating systems—whether proprietary like Windows or open-source like Linux—carry over 40 years of historical baggage:

- **文件系统** 诞生于磁盘需要人工管理目录的时代  
- **进程模型** 诞生于分时系统的共享计算时代  
- **系统调用** 诞生于硬件性能极其有限的时代  
- **安全模型** 建立在“管理员 vs 普通用户”的二元划分上  

- **File systems** were born in an era when disks required manual directory management  
- **Process models** were born in the era of time-sharing systems  
- **System calls** were born when hardware performance was extremely limited  
- **Security models** are built on the binary division of “administrator vs. ordinary user”

这些设计在当时是合理的，但今天它们已经成为：

These designs made sense at the time, but today they have become:

- 性能瓶颈（系统调用、TLB 刷新、数据拷贝）  
- 安全漏洞（内存安全、权限提升、侧信道攻击）  
- 用户体验的障碍（C 盘满了、后台进程、强制重启）

- Performance bottlenecks (system calls, TLB flushes, data copying)  
- Security vulnerabilities (memory safety, privilege escalation, side-channel attacks)  
- UX barriers (C: drive full, background processes, forced restarts)

**我们不需要“改进”这些设计，我们需要重新设计。**  
**We don't need to "improve" these designs—we need to redesign them.**

---

## 二、三大核心设计哲学  
## II. Three Core Design Philosophies

### 1. 彻底统一编址  
### 1. Unified 64‑bit Addressing

所有资源——内存、I/O 设备、文件、网络——全部映射到一个 **64 位虚拟地址空间**。

All resources—memory, I/O devices, files, network—are mapped into a **single 64‑bit virtual address space**.

- 数据不再在内核态和用户态之间“复制”  
- 文件不再需要“打开—读取—关闭”的路径  
- 网络数据包不再经过“协议栈—内核缓存—用户缓冲区”的三重拷贝

- Data is no longer “copied” between kernel and user space  
- Files no longer require an “open‑read‑close” path  
- Network packets no longer go through a triple copy (stack → kernel cache → user buffer)

**结果**：零拷贝数据流，无 TLB 刷新，性能接近裸机。  
**Result**: Zero‑copy data flow, no TLB flushes, performance close to bare metal.

---

### 2. 语言即是隔离墙  
### 2. Language as the Isolation Wall

用 Rust 的类型系统、所有权和借用检查，**替代 MMU 硬件隔离**。

Use Rust’s type system, ownership, and borrowing to **replace MMU hardware isolation**.

- 安全不靠页表，靠编译期检查  
- 上下文切换不切换页表，只保存/恢复寄存器  
- 即使单地址空间，Rust 也能保证内存安全

- Security relies not on page tables, but on compile‑time checks  
- Context switches do not switch page tables—only save/restore registers  
- Even in a single address space, Rust guarantees memory safety

**结果**：上下文切换极轻量，没有段错误，没有内核 panic。  
**Result**: Extremely lightweight context switches, no segmentation faults, no kernel panics.

---

### 3. 能力授权引擎  
### 3. Capability Authorization Engine

一切资源访问的唯一凭证，是不可伪造的**能力令牌**。

The only credential for accessing any resource is an **unforgeable capability token**.

- 程序不靠“拥有更多权限”，而靠“能证明自己应该被允许”  
- 令牌不可复制、不可绕过、可追溯  
- 没有“管理员”账号，没有“超级用户”

- Programs are not judged by “having more privileges” but by “proving they should be allowed”  
- Tokens cannot be copied, bypassed, or tampered with—they are fully traceable  
- There is no “administrator” account and no “superuser”

**结果**：最小权限原则天然实现，攻击面极小。  
**Result**: The principle of least privilege is naturally enforced, attack surface is minimal.

---

## 三、颠覆你对操作系统的认知  
## III. Redefining Your Understanding of Operating Systems

| 传统设计 | 新型设计 |
|---------|---------|
| 文件系统路径 | **键值对象存储**（哈希寻址，无目录树） |
| Socket / pipe | **发布-订阅 + 请求-响应通道** |
| 进程 / 线程 | **执行上下文**（寄存器 + 能力表，无页表切换） |
| 系统调用（int / syscall） | **直接函数调用**（`use_token()`） |
| 后台进程 | **完全透明运行状态**（无隐藏活动） |
| 强制重启更新 | **原子替换，永不强制重启** |
| 盘符 / 分区 | **全局存储池**（无 C 盘、D 盘） |
| 命令行 | **OmniBar 智能交互栏**（输入即所得） |

| Traditional Design | New Design |
|---------|---------|
| File system paths | **Key‑value object storage** (hash‑based addressing, no directory tree) |
| Socket / pipe | **Publish‑subscribe + request‑response channels** |
| Process / thread | **Execution context** (registers + capability table, no page‑table switch) |
| System calls (int / syscall) | **Direct function calls** (`use_token()`) |
| Background processes | **Fully transparent runtime state** (no hidden activity) |
| Forced‑restart updates | **Atomic replacement, never force a reboot** |
| Drive letters / partitions | **Global storage pool** (no C:, D:) |
| Command line | **OmniBar smart interaction bar** (type‑to‑action) |

---

## 四、安全与性能：从设计上保证  
## IV. Security and Performance: Guaranteed by Design

### 安全 · Security

- **内存安全**：Rust 编译期消除内存漏洞  
- **权限安全**：能力令牌是唯一访问凭证  
- **硬件兜底**：MPK + IOMMU 提供硬件级隔离  
- **无网络攻击面**：网络服务作为独立用户态服务，默认不开启

- **Memory safety**: Rust eliminates memory vulnerabilities at compile time  
- **Permission security**: Capability tokens are the sole access credential  
- **Hardware fallback**: MPK + IOMMU provide hardware‑level isolation  
- **No network attack surface**: Network services run as independent user‑space services, disabled by default

**设计承诺**：没有内存安全漏洞，没有段错误，系统永不因用户态程序崩溃。  
**Design commitment**: No memory safety vulnerabilities, no segmentation faults, the system never crashes due to user‑space program errors.

---

### 性能 · Performance

- **无系统调用**：直接函数调用，无特权级切换  
- **无 TLB 刷新**：全局单一页表，永不变更  
- **零拷贝**：数据在统一地址空间中直接传递  
- **确定性调度**：无随机延迟，缓存友好

- **No system calls**: Direct function calls, no privilege‑level switching  
- **No TLB flushes**: A single global page table, never changed  
- **Zero copy**: Data flows directly within the unified address space  
- **Deterministic scheduling**: No random delays, cache‑friendly

**实测目标**：开机 3 秒，关机 1 秒，上下文切换 < 0.01 ms，整体性能达到裸机 95% 以上。  
**Target benchmarks**: Boot in 3s, shutdown in 1s, context switch < 0.01 ms, overall performance above 95% of bare metal.

---

## 五、生态兼容：不妥协的兼容策略  
## V. Ecosystem Compatibility: A Non‑Compromising Approach

我们不兼容 POSIX，但我们兼容旧数据和应用：

We do not emulate POSIX, but we are compatible with legacy data and applications:

- **NTFS / FAT32 / EXT4**：可导入键值存储，不丢失用户数据  
- **WASI / WASM**：任何能编译到 WASI 的程序都能原生运行  
- **C / C++**：提供轻量级运行时，标准库调用映射为能力令牌调用

- **NTFS / FAT32 / EXT4**: Can be imported into key‑value storage without losing user data  
- **WASI / WASM**: Any program that compiles to WASI runs natively  
- **C / C++**: A lightweight runtime maps standard library calls to capability token calls

**我们不需要“模拟”旧世界，我们让旧世界自愿迁移过来。**  
**We don’t need to “emulate” the old world—we let it voluntarily migrate to us.**

---

## 六、从零构建，为民用而生  
## VI. Built from Scratch, for the People

这个操作系统不是为服务器、不是为企业、不是为资本而设计的。  
它是为每一个普通用户设计的：

This operating system is not designed for servers, for corporations, or for capital.  
It is designed for every ordinary user:

- **开机 3 秒**，不需要等待  
- **关机 1 秒**，不需要确认  
- **更新原子替换**，不强制重启，不打断工作  
- **没有后台进程**，所有状态可见  
- **没有盘符**，不需要理解 C 盘、D 盘  
- **没有命令行**，所有交互通过图形界面完成

- **Boot in 3 seconds** — no waiting  
- **Shutdown in 1 second** — no confirmation  
- **Atomic updates** — no forced restarts, no interruption  
- **No background processes** — all states are visible  
- **No drive letters** — no need to understand C:, D:  
- **No command line** — all interactions happen through the graphical interface

**技术自主，人民主权。**  
**Technological sovereignty, popular sovereignty.**

---

> “不是谁拥有更多权限，而是谁能证明它应该被允许。”  
> **新型操作系统 · 从零构建 · 为民用而生**

> “Not who has more privileges, but who can prove they should be allowed.”  
> **New Operating System · Built from Scratch · For the People**

---

**协作者入口 · Contributor Entry**  
**GitHub**: https://github.com/wupeixing192/New-Operating-System  
**Gitee**: https://gitee.com/wupeixing192/New-Operating-System  
**QQ频道**: RenMinSystem64
