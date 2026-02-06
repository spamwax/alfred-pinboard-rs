# Alfred Chromium Bookmarks

一个基于 Rust 的 Alfred Workflow，用于快速搜索 Chromium 系浏览器书签，并支持标签管理与目录过滤。

## 功能

- 支持多浏览器书签发现：Chrome、Arc、Dia、Brave、Edge、Vivaldi、Chromium、Opera 等。
- 支持 Alfred Script Filter 输出，回车即打开 URL。
- 支持标签（tag）增删改查与多标签 AND 过滤。
- 支持目录匹配与过滤：支持多级目录（如 `work/project/docs`）和模糊层级匹配。
- 支持 SQLite + FTS5 快速检索，自动缓存书签索引。

## 仓库结构

- `src/`: Rust 主程序
- `res/chrome-workflow/`: 可打包的 Alfred workflow 模板
- `scripts/build_workflow.sh`: 构建并打包 `.alfredworkflow`
- `scripts/bootstrap_workflow.sh`: 从源码一键产出可安装 workflow
- `scripts/git_feature_flow.sh`: 分支开发 + PR 流程辅助脚本

## 快速开始（从原始仓库到可用 workflow）

1. 克隆并进入项目。

```bash
git clone <your-repo-url>
cd alfred-pinboard-rs
```

2. 一键测试 + 构建 + 打包。

```bash
./scripts/bootstrap_workflow.sh -- --version 0.1.0
```

3. 安装 workflow。

```bash
open dist/AlfredChromeBookmarks.alfredworkflow
```

4. 在 Alfred 里输入 `cb rust` 验证。

## 打包脚本

### `scripts/build_workflow.sh`

```bash
./scripts/build_workflow.sh --version 0.1.0
```

常用参数：

- `--profile release|debug`：构建配置（默认 `release`）
- `--output <path>`：输出文件路径
- `--template <path>`：workflow 模板目录
- `--binary <path>`：指定已构建二进制
- `--skip-build`：跳过构建，仅打包

兼容入口：

```bash
./create_alfred_workflow.sh --version 0.1.0
```

## 使用方式

### Alfred 关键词

- `cb <query>`：普通搜索
- `cbf <query>`：模糊搜索

### 目录过滤（重点）

支持两种方式：

1. 命令参数：

```bash
alfred-chrome-bookmarks search --folders "work/project" rust
```

2. 内联语法（推荐 Alfred 场景）：

```bash
alfred-chrome-bookmarks search "rust folder:work/project"
alfred-chrome-bookmarks search "tokio dir:backend/docs"
alfred-chrome-bookmarks search "in:ai/research"
```

规则：

- 目录过滤支持多级路径，分隔符使用 `/`。
- 目录过滤为 AND 关系，可组合多个目录条件。
- 层级匹配支持部分命中，例如 `proj` 可匹配 `project`。

### 标签管理

```bash
# 添加标签
alfred-chrome-bookmarks tag "https://doc.rust-lang.org" rust docs

# 删除标签
alfred-chrome-bookmarks untag "https://doc.rust-lang.org" rust

# 重命名标签
alfred-chrome-bookmarks rename docs reference

# 查看书签标签
alfred-chrome-bookmarks show "https://doc.rust-lang.org"

# 列出标签
alfred-chrome-bookmarks tags
```

## 浏览器支持

程序会自动扫描 macOS 下 `~/Library/Application Support` 的 Chromium 系书签路径，包含但不限于：

- Google Chrome（含 Beta/Dev/Canary）
- Arc
- Dia
- Brave（含 Beta/Nightly）
- Microsoft Edge（含 Beta/Dev/Canary）
- Vivaldi
- Chromium
- Opera（多个发行通道）

可通过环境变量强制指定书签文件：

```bash
export ALFRED_CHROME_BOOKMARKS_PATH="$HOME/Library/Application Support/Arc/Default/Bookmarks"
```

## Alfred 运行配置

workflow 中 `run.sh` 会按以下优先级查找二进制：

1. `BINARY_PATH`（workflow variable）
2. workflow 目录内 `./alfred-chrome-bookmarks`
3. `~/.local/bin/alfred-chrome-bookmarks`
4. `~/.cargo/bin/alfred-chrome-bookmarks`
5. `/usr/local/bin/alfred-chrome-bookmarks`
6. `/opt/homebrew/bin/alfred-chrome-bookmarks`

## 开发协作规范（禁止直接改 master）

要求：后续改动不要直接提交 `master/main`，统一走“分支 -> PR -> 合并”。

推荐流程：

```bash
# 1) 从默认分支拉最新并切新分支
./scripts/git_feature_flow.sh start feat/folder-filter

# 2) 开发 + 本地验证
cargo test

# 3) 推送分支
./scripts/git_feature_flow.sh push

# 4) 创建 PR（示例）
gh pr create --fill
```

查看当前状态：

```bash
./scripts/git_feature_flow.sh status
```

## 测试

```bash
cargo test
```

## License

MIT
