# Alfred Workflow 配置指南

## 目标

从本仓库源码生成并安装可用的 Alfred workflow（关键词 `cb` / `cbf`）。

## 1. 构建与打包

```bash
./scripts/bootstrap_workflow.sh -- --version 0.1.0
```

产物：`dist/AlfredChromeBookmarks.alfredworkflow`

## 2. 安装

```bash
open dist/AlfredChromeBookmarks.alfredworkflow
```

## 3. 验证

在 Alfred 中输入：

- `cb rust`
- `cbf rust`

## 4. 可选变量

在 workflow Variables 中可设置：

- `BINARY_PATH`: 手动指定二进制路径

也可在 shell 中设置：

- `ALFRED_CHROME_BOOKMARKS_PATH`: 强制指定书签文件路径

```bash
export ALFRED_CHROME_BOOKMARKS_PATH="$HOME/Library/Application Support/Arc/Default/Bookmarks"
```

## 5. 目录过滤示例

```bash
alfred-chrome-bookmarks search "rust folder:work/project"
alfred-chrome-bookmarks search --folders "ai/research" "transformer"
```

## 6. 常见问题

- 提示 `Binary not found`：检查 `BINARY_PATH` 或重新执行打包脚本。
- 搜索无结果：先运行 `alfred-chrome-bookmarks refresh` 重新索引。
