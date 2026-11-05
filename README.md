# rust-ai-agent

一个用 Rust 实现的 AI Agent 工程，围绕 ReAct（Think → Act → Observe）循环构建，包含工具调用、上下文压缩、MCP 工具接入、Podman 沙箱代码执行、任务记忆、向量检索与 GAIA 评测流水线。

> 本项目基于 [solenovex/rust-ai-agent](https://github.com/solenovex/rust-ai-agent) 二次开发，原始版权归原作者所有。

## 功能概览

- **ReAct 运行时**：单步执行 `think / act / step`，支持最大步数限制、最终答案工具与结构化输出。
- **工具系统**：统一的 `Tool` trait，支持 OpenAI function calling 定义、参数 Schema、执行确认与结果回调。
  内置工具包括计算器、网页搜索、文件读取/列表/删除/解压、图片/音频/PDF/表格读取、任务规划、反思、最终答案，以及通过 MCP 动态注册的外部工具。
- **上下文管理**：`before_llm` 回调可在 token 超阈值时执行滑动窗口、摘要或压缩；`after_tool` 回调可对超长搜索结果做向量检索压缩。
- **沙箱代码执行**：可选启用 Podman 容器，提供持久会话的 `execute_python`、`bash_tool` 与受限目录的 `upload_file`，宿主机可通过桥接协议向沙箱 Python 暴露白名单函数。
- **MCP 集成**：以子进程方式启动 `wikipedia_mcp_server`，自动发现并注册其工具。
- **会话与记忆**：会话/事件模型与 `SessionManager` 抽象（默认内存实现）；任务记忆使用 SQLite 存储，结合 embedding 与余弦相似度召回，并带 LLM 查重。
- **检索增强**：定长分块、文本 embedding、余弦相似度与向量检索，用于搜索结果压缩。
- **GAIA 评测**：下载 GAIA 数据集与附件，扫描数据集并对比「无工具 / 有工具」两种模式的准确率。
- **技能发现**：扫描 `SKILL.md` 的 YAML frontmatter，仅将名称与描述注入提示词，正文按需读取。

## 运行流程

```
Agent::run
  ├─ 载入/创建 Session，追加 user 事件
  ├─ step
  │   ├─ think：BeforeLlm 回调 → 组装消息与工具定义 → 调用 LLM
  │   ├─ act：BeforeTool 回调 → 执行工具（含确认门控）→ AfterTool 回调
  │   └─ 判断是否命中 final_answer
  └─ 未完成则循环，直到完成或达到 max_steps
```

工具执行若需要人工确认，会返回 `AgentStatus::PendingConfirmation` 并保存挂起的工具调用；调用方带着确认结果再次进入 `run` 即可继续。

## 目录结构

```
.
├── src/
│   ├── main.rs                 # 示例入口：构建工具箱并运行一次 Agent
│   ├── lib.rs                  # 库入口，导出各模块
│   ├── constant.rs             # 环境变量名、模型名、Agent 名等常量
│   ├── agent/                  # Agent 模型、运行时（think/act/step）、事件、上下文、回调、会话确认
│   ├── llm/                    # async-openai 封装、请求/回答结构、并发信号量
│   ├── tool/                   # Tool trait、内置工具、MCP 客户端、沙箱代码工具
│   ├── sandbox/                # Podman 沙箱、桥接协议、工作区路径、配置解析、Python runner
│   ├── memory/                 # 任务记忆提取/存储/查重、提示词
│   ├── search/                 # 分块、embedding、相似度、向量检索
│   ├── planning/               # 任务规划数据结构
│   ├── session/                # 会话模型与会话管理器
│   ├── gaia/                   # 数据集下载、求解器、评测器
│   ├── skill.rs                # SKILL.md 技能发现
│   └── bin/
│       ├── gaia.rs             # GAIA 评测入口
│       └── wikipedia_mcp_server.rs  # Wikipedia MCP 服务端
├── examples/
│   └── web_rag.rs              # 网页搜索 + 分块 + 向量检索示例
├── sandbox/
│   ├── Containerfile           # 沙箱镜像定义（Python 3.12 + pandas/numpy/pypdf 等）
│   └── runner.py               # 沙箱内执行协议实现
├── .env.example                # 环境变量示例
├── Cargo.toml
└── README.md
```

## 环境要求

- Rust 1.93+（`edition = 2024`）
- 一个 OpenAI 兼容的 API 端点与密钥
- 可选：Podman（启用代码执行沙箱时需要）
- 可选：`libpdfium` 动态库（解析 PDF 时使用；默认从 `CARGO_MANIFEST_DIR` 查找，或通过 `PDFIUM_LIB_PATH` 指定）

## 配置

复制 `.env.example` 为 `.env` 并按需填写：

| 变量 | 说明 |
| --- | --- |
| `OPENAI_API_KEY` | OpenAI 兼容服务的 API Key |
| `OPENAI_BASE_URL` | API 基地址，示例中为 OpenRouter |
| `HF_TOKEN` | Hugging Face Token，下载 GAIA 数据集时需要 |
| `TAVILY_API_KEY` | Tavily 搜索 Key，`web_search` 工具与示例需要 |
| `CODE_EXECUTION` | 设为 `podman` 启用沙箱；未设置或 `off` 表示关闭 |
| `SANDBOX_WORKSPACE` | 挂载到沙箱 `/workspace` 的宿主机目录，默认 `./workspace` |
| `UPLOAD_ROOT` | 允许上传到沙箱的宿主机目录；不设置则不注册 `upload_file` |
| `SKILLS_DIR` | 技能目录，只读挂载到沙箱 `/skills` |
| `PDFIUM_LIB_PATH` | pdfium 动态库路径 |

## 构建与运行

```bash
cargo check
```

运行示例 Agent（不带参数时执行内置的沙箱计算示例）：

```bash
cargo run -- "请调研 Rust 异步运行时的工作窃取调度机制"
```

启用代码执行沙箱：

```bash
podman build -t localhost/ai-agent-sandbox:latest -f sandbox/Containerfile sandbox/
# Linux/macOS
CODE_EXECUTION=podman cargo run -- "用 Python 计算 1 到 100 的平方和，并用 bash 验证"
# PowerShell
$env:CODE_EXECUTION="podman"; cargo run -- "用 Python 计算 1 到 100 的平方和，并用 bash 验证"
```

运行 GAIA 评测（会下载数据集与附件到 `gaia_cache/`）：

```bash
cargo run --bin gaia
```

单独运行 Wikipedia MCP 服务：

```bash
cargo run --bin wikipedia_mcp_server
```

运行检索示例：

```bash
cargo run --example web_rag
```

注意：主流程会以子进程方式执行 `cargo run --quiet --bin wikipedia_mcp_server` 来接入 MCP 工具，因此需要 `cargo` 在 `PATH` 中可用。

## 主要依赖

| 依赖 | 用途 |
| --- | --- |
| `tokio` | 异步运行时 |
| `async-openai` | OpenAI 兼容 API 客户端 |
| `rmcp` | MCP 客户端/服务端与子进程传输 |
| `serde` / `serde_json` / `schemars` | 序列化与工具参数 Schema |
| `reqwest` | HTTP 请求（搜索、Wikipedia、数据集） |
| `rusqlite` | SQLite 任务记忆存储 |
| `tiktoken-rs` | 上下文 token 计数 |
| `backon` | 重试 |
| `calamine` / `csv` / `pdfium-render` / `image` | 表格、CSV、PDF、图片处理 |
| `zip` | 附件解压 |
| `tracing` / `tracing-subscriber` | 日志 |
| `dotenv` | `.env` 加载 |
| `uuid` / `chrono` | ID 与时间 |

## 说明

- 默认模型名在 `src/constant.rs` 中定义（`openai/gpt-6-luna-pro`），需与所用 API 端点支持的模型一致。
- 默认 `max_steps` 为 10，可在 `main.rs` 中通过 `with_max_steps` 调整。
- `memory_tool` 与记忆注入回调以库模块形式提供，示例入口默认未启用。

## 版权与出处

本项目基于 [solenovex/rust-ai-agent](https://github.com/solenovex/rust-ai-agent) 二次开发，原始版权归原作者所有。

上游仓库未附带 `LICENSE` 文件；如需再分发或商用，请先与原作者确认授权方式。
