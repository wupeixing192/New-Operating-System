欢迎参与新型操作系统的功能开发！本目录包含了所有用户态服务模块的**空壳框架**。  
Welcome to the feature development of the New Operating System! This directory contains the **empty skeleton** for all user‑space service modules.

你可以自由选择其中一个模块，按照接口定义实现具体功能。接口由你自行设计，框架只提供目录结构和依赖配置，不限制实现方式。  
You are free to pick any module and implement its functionality according to the interface you define. The framework only provides the directory structure and dependency configuration—it does not constrain how you implement it.

---

## 目录结构 · Directory Structure

```text
功能-Function/
├── 键值存储服务-Key-Value Storage Service/
│ ├── 接口-interface/ # 由你定义 trait 和类型 / Define your trait and types here
│ │ └── main.rs # （空文件，等待你的设计）(Empty, awaiting your design)
│ └── 主内容-Main Content/ # 由你实现具体逻辑 / Implement your logic here
│ ├── Cargo.toml # 已配置好依赖（引用 ../接口-interface）
│ └── src/
│ └── lib.rs # （空文件，等待你的实现）(Empty, awaiting your implementation)
├── 网络服务-Internet services/
│ └── ...（同上）
├── 图形\Vulkan服务-Graphics\Vulkan Service/
│ └── ...（同上）
├── OmniBar 智能交互栏/
│ └── ...（同上）
├── AI助手服务-AI Assistant Service/
│ └── ...（同上）
└── 看护服务-Care services/
└── ...（同上）
```
---
每个模块都分为两个部分：  
Each module is divided into two parts:

- **接口文件夹**（`接口-interface/`）：定义该模块的公开 API（trait、类型、常量等）。这是给其他模块和内核使用的契约。  
- **主文件夹**（`主内容-Main Content/`）：实现具体的功能逻辑，依赖接口文件夹中的定义。

- **Interface folder** (`接口-interface/`): Define the public API of the module (traits, types, constants, etc.). This is the contract used by other modules and the kernel.  
- **Main content folder** (`主内容-Main Content/`): Implement the actual functionality, depending on the definitions in the interface folder.

---

## 如何开始 · How to Get Started

### 1. 选择模块 · Pick a Module
浏览上面的目录，确定你感兴趣的服务（例如“键值存储服务”）。  
Browse the directories above and choose a service that interests you (e.g., “Key‑Value Storage Service”).

### 2. 设计接口 · Design the Interface
进入该模块的 `接口-interface/` 目录，打开 `main.rs`（或新建 `lib.rs`），定义你的 trait 和数据结构。例如：  
Go to the `接口-interface/` folder of that module, open `main.rs` (or create `lib.rs`), and define your trait and data structures. For example:

```rust
// 键值存储服务接口示例 / Example interface for Key‑Value Storage
pub trait StorageService {
    fn create_object(&mut self, data: &[u8]) -> Result<u64, &'static str>;
    fn read_object(&mut self, id: u64) -> Result<Vec<u8>, &'static str>;
    // ...
}
```
接口应保持稳定，因为其他模块会依赖它。
Keep the interface stable, as other modules will depend on it.

### 3. 实现功能 · Implement the Functionality
进入 主内容-Main Content/ 目录，在 src/lib.rs 中实现你的 trait。你可以引用接口文件夹中的定义（已在 Cargo.toml 中配置好路径）。
Go to the 主内容-Main Content/ folder, implement your trait in src/lib.rs. You can reference the definitions from the interface folder (the path is already configured in Cargo.toml).

```rust
use storage_interface::StorageService;

pub struct MyStorage {
    // ...
}

impl StorageService for MyStorage {
    // 实现所有方法 / Implement all methods
}
```
### 4. 本地测试 · Test Locally
在主文件夹内执行 cargo build 确保编译通过。你可以添加单元测试和示例代码。
Run cargo build inside the main folder to ensure it compiles. You may add unit tests and example code.

### 5. 提交代码 · Submit Your Code
完成实现后，提交 Pull Request（PR）到 main 分支。PR 中请简要说明你实现的功能和接口设计。
After finishing, submit a Pull Request (PR) to the main branch. Briefly describe the functionality you implemented and your interface design in the PR.

注意事项 · Notes
接口设计：尽量保持接口简洁、清晰。如有必要，提供文档注释。

依赖管理：你可以在主文件夹的 Cargo.toml 中添加额外依赖，但请确保它们不会与其他模块冲突。

代码风格：建议使用 Rust 官方风格（cargo fmt）。

沟通：如果你不确定某个接口设计，可以在群里讨论。

Interface design: Keep it simple and clear. Provide doc comments if necessary.

Dependency management: You may add extra dependencies in the main folder’s Cargo.toml, but ensure they do not conflict with other modules.

Code style: Please use the official Rust style (cargo fmt).

Communication: If you are unsure about an interface design, feel free to discuss it in the group.

## 常见问题 · FAQ
Q: 我可以同时开发多个模块吗？
Can I work on multiple modules at the same time?
A: 当然可以，但建议先完成一个，再开始下一个，以便集中精力。
A: Yes, but it’s recommended to finish one before starting another to stay focused.

Q: 如果接口需要修改，怎么办？
What if the interface needs to be changed?
A: 接口修改可能影响其他模块，请先与团队沟通，确保变更合理。
A: Interface changes may affect other modules—please communicate with the team first to ensure the change is reasonable.

Q: 主文件夹中的 Cargo.toml 已经引用 ../接口-interface，为什么是相对路径？
Why does the main folder’s Cargo.toml reference ../接口-interface as a relative path?
A: 这样便于本地开发时直接引用接口定义，同时未来可以迁移为独立 crate。
A: This allows local development to directly reference the interface definitions, and in the future it can be migrated to an independent crate.

## 欢迎加入开发！如有任何问题，请在 QQ 频道或 GitHub Issues 中提出。
Welcome to the development! For any questions, please ask in the QQ channel or GitHub Issues.
