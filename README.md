# Alfred Chrome Bookmarks 书签管理工作流

一个强大的Alfred工作流，用于管理Chrome书签并支持多标签(tag)维度搜索。

## ✨ 功能特性

- 📖 **读取Chrome本地书签** - 直接访问Chrome书签文件，无需额外导出
- 🏷️ **多标签系统** - 为书签添加多个自定义标签
- 🔍 **强大的搜索** - 支持按标题、URL、标签进行搜索
- ⚡ **模糊搜索** - 使用模糊匹配算法快速找到书签
- 🎯 **多标签过滤** - 同时按多个标签进行AND查询
- 💾 **本地存储** - 标签信息存储在本地SQLite数据库中

## 📦 安装

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/your-username/alfred-chrome-bookmarks.git
cd alfred-chrome-bookmarks

# 构建release版本
cargo build --release

# 二进制文件位于: target/release/alfred-chrome-bookmarks
```

## 🚀 使用方法

### 在Alfred中搜索书签

设置Alfred Workflow后，直接在Alfred中输入关键词搜索：

```
cb github rust
```

### 命令行使用

#### 搜索书签

```bash
# 基础搜索
./alfred-chrome-bookmarks search rust

# 按标签过滤 (AND查询)
./alfred-chrome-bookmarks search --tags work,dev rust

# 使用模糊搜索
./alfred-chrome-bookmarks search --fuzzy githb

# 限制结果数量
./alfred-chrome-bookmarks search --limit 10 javascript
```

#### 标签管理

```bash
# 添加标签
./alfred-chrome-bookmarks tag "https://github.com/rust-lang/rust" rust programming

# 列出所有标签
./alfred-chrome-bookmarks tags

# 搜索特定标签
./alfred-chrome-bookmarks tags work

# 显示书签的标签
./alfred-chrome-bookmarks show "https://github.com/rust-lang/rust"

# 删除标签
./alfred-chrome-bookmarks untag "https://github.com/rust-lang/rust" rust

# 重命名标签
./alfred-chrome-bookmarks rename old-tag new-tag
```

#### 其他命令

```bash
# 刷新书签缓存
./alfred-chrome-bookmarks refresh

# 显示统计信息
./alfred-chrome-bookmarks stats
```

## ⚙️ 配置

### Chrome书签路径

默认情况下，程序会自动查找Chrome书签文件：

- macOS: `~/Library/Application Support/Google/Chrome/Default/Bookmarks`
- 支持多个Profile: `Profile 1`, `Profile 2` 等

### 数据存储

标签数据存储在：

- Alfred环境: `$alfred_workflow_data/tags.db`
- 非Alfred环境: `~/.alfred-chrome-bookmarks/tags.db`

## 🎨 在Alfred中设置工作流

1. 打开Alfred Preferences → Workflows
2. 点击 + → Blank Workflow
3. 添加Script Filter:
   - Keyword: `cb`
   - Script: `/path/to/alfred-chrome-bookmarks search {query}`
   - Title: `{title}`
   - Subtitle: `{subtitle}`
   - Arg: `{arg}`
4. 添加Open URL动作，连接到Script Filter，URL设置为 `{query}`
5. （可选）添加其他命令的Script Filter

示例Alfred workflow配置见下方。

## 🛠️ 技术栈

- **语言**: Rust 2021
- **数据库**: SQLite (rusqlite)
- **模糊搜索**: fuzzy-matcher
- **Alfred集成**: alfred-rs

## 📝 命令行接口

```bash
alfred-chrome-bookmarks <SUBCOMMAND>

子命令:
    search, s      搜索书签
    tag, t         为书签添加标签
    untag, ut      从书签删除标签
    tags, lt       列出所有标签
    show, sh       显示书签的标签
    rename, r      重命名标签
    refresh, rf    刷新Chrome书签缓存
    stats, st      显示统计信息
```

## 💡 使用案例

### 场景1: 组织工作书签

```bash
# 标记工作相关书签
./alfred-chrome-bookmarks tag "https://jira.company.com" work project-a
./alfred-chrome-bookmarks tag "https://confluence.company.com" work documentation

# 在Alfred中快速搜索: cb --tags work
```

### 场景2: 学习资源管理

```bash
# 标记学习资源
./alfred-chrome-bookmarks tag "https://rust-lang.org" rust learning tutorial
./alfred-chrome-bookmarks tag "https://doc.rust-lang.org" rust documentation

# 查找所有Rust学习资源: cb --tags rust,learning
```

### 场景3: 项目管理

```bash
# 为不同项目添加标签
./alfred-chrome-bookmarks tag "https://github.com/project-a" github project-a
./alfred-chrome-bookmarks tag "https://github.com/project-b" github project-b

# 查看特定项目: cb --tags project-a
```

## 🔧 故障排除

### Chrome书签文件未找到

确保Chrome已经安装，或者检查书签文件路径是否正确。

### 数据库错误

如果遇到数据库错误，可以删除数据库文件重新开始：

```bash
rm ~/.alfred-chrome-bookmarks/tags.db
```

### 搜索结果为空

1. 确保Chrome书签文件存在且可读
2. 尝试使用模糊搜索 (`--fuzzy`)
3. 检查标签拼写是否正确

## 📊 性能

- 书签加载: 毫秒级
- 搜索响应: 毫秒级 (基于索引)
- 数据库大小: 轻量级 (< 1MB，取决于标签数量)

## 🔮 未来计划

- [ ] 支持Firefox书签
- [ ] 书签自动同步
- [ ] 标签自动建议
- [ ] 书签导入/导出
- [ ] Web界面
- [ ] 书签分析和统计

## 📄 许可证

本项目采用MIT许可证。

---

**享受更高效的书签管理体验！** 🚀
