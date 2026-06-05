

贡献指南/Contribution Guide

- 报告 Bug / Bug reports
- 文档改进 / Documentation improvements
- 架构讨论 / Architecture discussions
- 代码片段建议 / Code snippet suggestions
- 安全审计与审查 / Security audits and reviews
- 兼容驱动开发 / Driver compatibility
- 网络栈开发 / Network stack
- 键‑值对象存储服务 / Key‑value object storage services
- 发布‑订阅通道模块 / Pub‑sub channel modules
- 用户态应用 / User‑space applications
- 文档改进 / Documentation improvements

不接受的贡献 / What We Do Not Accept

- **内核代码修改：本项目内核（包含引导程序、能力授权引擎、物理内存分配器、调度器等核心模块）由项目创始人独立编写与维护，不接受外部贡献。**
- **Kernel code modifications: The kernel (including the bootloader, capability token engine, physical memory allocator, scheduler, and other core modules) is written and maintained exclusively by the project founder. External contributions to the kernel are not accepted.**
- 违反核心架构的修改（例如引入 POSIX 兼容）
- Changes that violate the core architecture (e.g., introducing POSIX compatibility)
- 试图绕过能力令牌模型的贡献
- Contributions that attempt to bypass the capability token model
- 任何削弱"语言即隔离墙"原则的修改
- Any modification that weakens the "language-as-isolation-wall" principle

核心原则 / Core Principle

本项目的架构和核心设计由创始人定义。所有贡献者必须在此框架内协作。

The architecture and core design of this project are defined by the founder. All contributors must collaborate within this framework.

贡献前须知 / Before You Contribute

1. 阅读 `ARCHITECTURE.md`（如果有）以理解设计哲学。
1. Read the `ARCHITECTURE.md` (if available) to understand the design philosophy.
2. 在提交 Pull Request 前，先开 Issue 讨论重大修改。
2. Open an Issue to discuss major changes before submitting a Pull Request.
3. 保持贡献的专注和最小化。
3. Keep contributions focused and minimal.

许可证 / License

通过贡献，你同意你的贡献将按照本项目的许可证进行授权。

By contributing, you agree that your contributions will be licensed under the same license as this project.
