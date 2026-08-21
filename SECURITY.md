# 安全策略 / Security Policy

## 项目声明 / Project Statement

**新型操作系统**是一个从零构建的操作系统项目，安全是其核心设计原则之一。
本项目的三大安全支柱：

1. **彻底统一编址** — 消除传统内核/用户空间边界带来的攻击面
2. **Rust 编译期隔离** — 用类型系统和借用检查器替代 MMU，从语言层面消灭整类漏洞
3. **能力授权引擎** — 所有资源访问必须持有不可伪造的能力令牌，无令牌即无权限

The New Operating System is built from scratch with security as a core design
principle. Its three security pillars: unified addressing, Rust compile-time
isolation, and a capability-based authorization engine.

---

## 支持的版本 / Supported Versions

本项目目前处于**早期开发阶段**，主分支（`main`）为唯一受支持的分支。

| 版本 | 支持状态 |
|------|----------|
| main（最新提交） | ✅ 完全支持 |
| 其他分支 | ⚠️ 仅开发用途，不保证安全更新 |

> 项目尚未发布正式版本（0.x/1.0 待定）。首个稳定版本发布后，本表将更新为具体版本号。

This project is in **early development**. The `main` branch is the only
supported branch. A formal versioned security support policy will be
established upon the first stable release.

---

## 漏洞上报 / Reporting a Vulnerability

### ⚠️ 重要：请勿公开披露

**不要**通过公开 GitHub Issue 报告安全漏洞。
公开披露会给用户带来不必要的风险。请通过以下**私密渠道**报告。

**Do NOT** report security vulnerabilities via public GitHub Issues.
Please use the **private channels** below.

### 上报渠道 / Reporting Channels

| 渠道 | 方式 |
|------|------|
| **QQ 频道（首选）** | RenMinSystem64 — 私信项目维护者 |
| **GitHub Security Advisory** | 访问仓库 → Security 标签页 → Report a vulnerability |
| **加密邮件** | 待补充（首个稳定版发布前公布） |

### 上报内容模板 / Report Template

请尽可能包含以下信息：

```
1. 漏洞类型（如：能力令牌伪造、权限提升、内存越界等）
2. 受影响的内核模块或子系统
3. 复现步骤（最小可复现代码/配置）
4. 攻击前提条件（需要何种能力令牌/权限级别）
5. 影响评估（可读取/写入/执行的范围）
6. 建议修复方案（如有）
```

### 响应时效 / Response Timeline

| 阶段 | 目标时间 |
|------|----------|
| 确认收到报告 | ≤ 72 小时 |
| 初步评估与分类 | ≤ 7 天 |
| 修复开发与测试 | 视漏洞复杂度而定 |
| 发布安全公告 | 修复合并后立即发布 |

| Phase | Target |
|-------|--------|
| Acknowledgment | ≤ 72 hours |
| Initial assessment | ≤ 7 days |
| Fix development & testing | Varies by complexity |
| Security advisory publication | Immediately after merge |

---

## 安全设计原则 / Security Design Principles

### 能力令牌（Capability Token）

- 一切资源访问的唯一凭证
- 不可伪造、不可猜测、可撤销
- 任何绕过能力令牌直接操作物理地址的行为，视为对系统安全模型的根本破坏

### 内核/用户态隔离

- 本项目**不使用传统 MMU 页表隔离**
- 依赖 Rust 类型系统 + 所有权模型实现编译期隔离
- 上下文切换开销极低，安全不牺牲性能

### GPU 直通

- 内核原生支持基于 VT-d 的 GPU 直通
- 向用户态提供 GPU 能力令牌，而非显卡抽象层
- 图形性能不低于裸机水平的 95%

---

## 安全更新流程 / Security Update Process

1. 漏洞确认后，在私有分支上开发修复
2. 编写回归测试用例，确保漏洞不再复现
3. 通过 GitHub Security Advisory 私下协调披露
4. 修复合并到 `main` 分支
5. 发布安全公告（含 CVE 编号，如适用）
6. 更新 CHANGELOG.md 中的安全条目

---

## 密钥与凭证管理 / Key & Credential Management

- 本项目不依赖任何长期有效的对称密钥
- 能力令牌由内核在运行时动态生成，重启后失效
- 构建系统不嵌入任何硬编码密钥或凭证
- CI/CD 密钥通过 GitHub Secrets 管理，不进入源码

---

## 已知安全限制 / Known Security Limitations

| 限制 | 说明 | 缓解措施 |
|------|------|----------|
| 早期阶段代码审计不充分 | 项目处于开发早期，未经第三方安全审计 | 欢迎安全研究者参与审查 |
| 无正式发布版本 | 无 CVE 编号体系对接 | 通过本文件渠道报告 |
| 硬件安全依赖 | 信任 VT-d/IOMMU 等硬件机制 | 在合规硬件上运行 |

---

## 免责声明 / Disclaimer

本文件仅供项目参与者参考，不构成安全保证或法律建议。
操作系统安全是一个持续过程，无法做出绝对安全的承诺。

This document is for reference only and does not constitute a security
guarantee or legal advice. OS security is an ongoing process; no absolute
security can be guaranteed.

---

## 联系方式 / Contact

- **QQ 频道**：RenMinSystem64
- **GitHub**：https://github.com/wupeixing192/New-Operating-System
- **Issues**：https://github.com/wupeixing192/New-Operating-System/issues

---

© 2026-2027 新型操作系统开发者团队 · GPL-3.0-or-later

