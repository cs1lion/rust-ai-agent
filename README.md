# rust-ai-agent
![Rust 2021](img.shields.io/badge/Rust-2021-blue) ![Sandbox Podman](img.shields.io/badge/Sandbox-Podman-blue) ![Benchmark GAIA](img.shields.io/badge/Benchmark-GAIA-blue)

基于 Rust 构建的高性能、类型安全的 AI Agent 框架，实现 ReAct 循环、上下文工程、MCP 工具生态、Podman 沙箱隔离与 GAIA Benchmark 自动化评测。

> 本项目基于 [solenovex/rust-ai-agent](https://github.com/solenovex/rust-ai-agent) 二次开发。

## 核心技术深度 (Technical Highlights)

- **ReAct Runtime (`think/act/step`)**：单步解耦的 `think / act / step` 循环机制，支持最大步数限制、人机确认门控（`PendingConfirmation`）及结构化 JSON/Final Answer 导出。
- **Context Engineering**：
  - `before_llm`：在 Token 超限时触发滑动窗口压缩与摘要生成。
  - `after_tool`：对超长工具/搜索输出进行向量切块与语义压缩。
  - **SQLite Embedding 记忆召回**：基于 SQLite + 余弦相似度的长期记忆向量检索与 LLM 重排。
- **Tool & MCP Ecosystem**：
  - 统一 `Tool` Trait：支持 OpenAI Function Calling 规范及参数 Schema 校验。
  - **MCP 自动发现**：以子进程方式启动 MCP Server（如 `wikipedia_mcp_server`），自动发现并注册工具。
  - **`SKILL.md` 动态技能注入**：扫描 YAML Frontmatter 检索技能声明，按需动态装载正文提示词。
- **Podman 容器安全代码沙箱**：基于 Podman 容器提供会话级持久化隔离环境，支持 `execute_python`、`bash_tool` 及受限文件上传下载。
- **GAIA Benchmark 评测流水线**：自动下载 GAIA 评估数据集与附件，执行并对比无工具 vs 有工具模式的端到端准确率。

## 目录结构 (Directory Structure)

```
.
├── src/
│   ├── agent/       # ReAct Runtime 核心驱动与 step 控制
│   ├── context/     # 滑动窗口压缩与向量上下文工程
│   ├── tools/       # 统一 Tool Trait, Podman 沙箱与 MCP 接入
│   ├── memory/      # SQLite Embedding 检索与会话持久化
│   └── eval/        # GAIA Benchmark 自动化评测逻辑
├── crates/          # 独立 crate 与 MCP 工具组件
└── tests/           # 单元测试与集成测试套件
```

## 快速开始 (Quick Start)

### 编译与运行

```bash
# 构建项目
cargo build --release

# 运行主 Agent 交互 CLI
cargo run --bin rust-ai-agent

# 运行 GAIA Benchmark 评测流水线
cargo run --bin gaia-eval
```

## Key Engineering Enhancements
- **Context Engineering Hooks**: `before_llm` sliding-window compression & `after_tool` vector chunk filtering.
- **Podman Container Sandbox**: Persistent isolated execution environment for `execute_python` and `bash_tool`.
- **MCP & Dynamic Skills**: Subprocess MCP tool auto-discovery and `SKILL.md` frontmatter injection.
- **GAIA Benchmark Harness**: Automated multi-step evaluation pipeline and telemetry export.
