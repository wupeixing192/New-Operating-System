# 更新日志 / Changelog

本文件记录新型操作系统的所有重要变更。
格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，
版本号遵循 [语义化版本 2.0.0](https://semver.org/lang/zh-CN/)。

This file records all notable changes to the New Operating System.
Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
versioning follows [Semantic Versioning 2.0.0](https://semver.org/).

---

## [Unreleased]

### 计划中 / Planned
- [ ] 内核基础内存管理（统一 64 位地址空间映射）
- [ ] 能力令牌引擎原型
- [ ] Rust 编译期隔离验证框架
- [ ] 基础引导加载程序（Bootloader）
- [ ] 最小化用户态运行时

---

## [0.0.1] - 2026-07-24

### Added / 新增
- 项目仓库初始化
- 完整 GPL-3.0-or-later 许可证文件（`LICENSE`、`COPYING`）
- `NOTICE` 文件（版权声明、许可证选择说明、贡献者须知、第三方组件声明）
- `CONTRIBUTING.md`（贡献指南，含 DCO 签署说明）
- `CODE_OF_CONDUCT.md`（社区行为准则，BDFL 治理模型）
- `SECURITY.md`（安全策略与漏洞上报流程）
- `ARCHITECTURE.md`（系统架构设计文档）
- `README.md`（项目介绍，中英文双语）
- 法律合规指引文档（`新型操作系统法律合规指引.docx`）
- 技术宪法文档（`制造新型操作系统技术宪法.docx`）
- 功能文档系列：
  - 存储体系与编译原理规范
  - Ring 3 硬件代码设计
  - 系统桌面开发
  - Vulkan 图形显示支持
  - 软件兼容性与多语言生态支持
- 图解操作系统演示文稿（`图解操作系统.pptx`）

### Design Decisions / 设计决策
- **许可证选择**：GPL-3.0-or-later（非 GPL-3.0-only），决策记录于 2026 年 7 月
- **贡献模式**：非内核功能接受社区贡献；内核由 BDFL 独立维护
- **安全模型**：永不引入 POSIX 语义；一切资源访问通过能力令牌
- **性能目标**：GPU 直通不低于裸机 95%；系统启动 ≤ 3 秒
- **更新机制**：原子替换，永不强制重启

### 核心架构理念 / Core Architectural Concepts
1. **彻底统一编址**：所有资源映射到单一 64 位虚拟地址空间
2. **语言即是隔离墙**：用 Rust 类型系统替代 MMU 硬件隔离
3. **能力授权引擎**：不可伪造令牌作为一切资源访问的唯一凭证

---

## 版本说明 / Version Notes

### 当前阶段
项目处于**早期设计与原型阶段**（Pre-Alpha）。
所有 API、内部接口、安全机制均可能发生重大变更。

### 版本号规则
- `0.x.y`：开发阶段，不保证稳定性
- `1.0.0`：首个稳定版本，ABI 冻结
- 重大架构变更将导致主版本号递增

### 修改标示要求（GPLv3 §5）
根据 GPLv3 要求，每次修改必须标示：
- **修改内容**：在对应源文件头部注明
- **修改日期**：更新文件头部版权年份范围
- **修改人身份**：通过 DCO Signed-off-by 行记录

---

## 许可证 / License

本项目采用 GNU General Public License v3.0 or later。
详情见 `LICENSE` 文件。

---

[Unreleased]: https://github.com/wupeixing192/New-Operating-System/compare/v0.0.1...HEAD
[0.0.1]: https://github.com/wupeixing192/New-Operating-System/releases/tag/v0.0.1

---

© 2026-2027 新型操作系统开发者团队 · GPL-3.0-or-later

