# Castor: Gemini Session Manager (Rust)

`castor` 是一个使用 Rust 编写的、安全优先的本地 Gemini 会话管理器。

## 1. 核心定位
- **安全 (Safety)**: 默认 Dry-run，软删除逻辑，操作审计。
- **透明 (Transparency)**: 结构化日志，实时预览。
- **现代 (Modern)**: 基于 Rust 强类型，利用 `ratatui` 提供流畅的 TUI 体验。

## 2. 目录结构 (Rust Idiomatic Layout)

```text
castor/
├── Cargo.toml              # 项目依赖与元数据
├── README.md               # 项目介绍
├── docs/                   # 详细文档
│   ├── ARCHITECTURE.md     # 架构设计
│   └── COMMANDS.md         # 命令手册
├── src/
│   ├── main.rs             # 程序入口，处理顶层错误
│   ├── cli.rs              # 命令行解析 (基于 clap)
│   ├── config.rs           # 配置文件加载与校验
│   ├── error.rs            # 自定义错误类型 (基于 thiserror)
│   ├── core/               # 领域核心逻辑
│   │   ├── mod.rs
│   │   ├── session.rs      # 会话模型与元数据解析
│   │   ├── scanner.rs      # 本地文件扫描器
│   │   └── registry.rs     # 会话注册表与索引
│   ├── ops/                # 原子操作
│   │   ├── mod.rs
│   │   ├── delete.rs       # 安全删除逻辑 (Dry-run/Soft/Hard)
│   │   ├── restore.rs      # 恢复逻辑
│   │   └── executor.rs     # 批量执行器
│   ├── audit/              # 审计与日志
│   │   ├── mod.rs
│   │   ├── logger.rs       # JSONL 格式审计记录
│   │   └── history.rs      # 历史记录回溯 (Batch ID)
│   ├── tui/                # 终端 UI 模块 (基于 ratatui)
│   │   ├── mod.rs
│   │   ├── app.rs          # TUI 状态管理
│   │   ├── ui.rs           # 界面渲染逻辑
│   │   ├── widgets/        # 自定义组件 (树形列表、预览框)
│   │   └── event.rs        # 事件循环 (按键、刷新)
│   └── utils/              # 通用工具函数
│       ├── mod.rs
│       ├── fs.rs           # 文件系统增强工具 (移动/复制)
│       └── term.rs         # 终端兼容性处理
└── tests/                  # 集成测试
    ├── common/             # 测试脚手架 (Fixtures)
    ├── cli_test.rs         # CLI 命令测试
    └── ops_test.rs         # 核心操作测试
```

## 3. 关键依赖 (Initial Crate Selection)

- **CLI**: `clap = { version = "4", features = ["derive"] }`
- **TUI**: `ratatui`, `crossterm`
- **Serialization**: `serde`, `serde_json`
- **Error Handling**: `thiserror`, `anyhow`
- **Async (Optional)**: `tokio` (如果需要并发扫描大量会话)
- **File Utilities**: `walkdir`, `fs_extra`
- **DateTime**: `chrono`
- **Config**: `directories` (符合各平台规范的配置路径)

## 4. 安全设计基准
1. **Dry-run First**: 所有修改状态的操作必须显式传递 `--confirm` 标志。
2. **Soft Delete**: 删除操作默认移动到 `~/.gemini/trash`。
3. **Atomic Audit**: 每次操作生成的 `batch_id` 必须先写入审计日志，再执行物理操作。
4. **Validation**: 启动时检查 Gemini 默认存储路径的有效性。
