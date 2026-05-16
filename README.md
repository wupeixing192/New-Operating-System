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

# 宣言/Declaration:：

内核必须原生支持基于VT-d的GPU直通能力，向用户态提供GPU能力令牌，而非显卡抽象层。图形性能不得低于裸机水平的95%。

本系统永不引入POSIX语义。任何绕过能力令牌直接操作物理地址的行为，均视为对系统安全模型的根本破坏。

系统启动时间（从按下电源到可交互）不超过3秒。应用安装不得修改全局状态，通过键值对象存储实现独立沙箱。系统更新采用原子替换，永不强制重启。

The kernel must natively support GPU pass through capabilities based on VT-d, providing GPU capability tokens to user mode rather than the graphics card abstraction layer. The graphics performance shall not be lower than 95% of the bare metal level.

This system will never introduce POSIX semantics. Any behavior that bypasses the ability token and directly operates on physical addresses is considered a fundamental breach of the system security model.

The system startup time (from pressing the power button to being interactive) shall not exceed 3 seconds. Application installation cannot modify the global state, and an independent sandbox is implemented through key value object storage. The system update uses atomic substitution and never forces a restart.
