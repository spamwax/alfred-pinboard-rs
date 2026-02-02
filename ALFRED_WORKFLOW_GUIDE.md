# Alfred Workflow 配置指南

本指南将帮助你在Alfred中设置Chrome书签管理workflow。

## 前提条件

1. 已安装Alfred (需要Powerpack许可证)
2. 已构建或下载了`alfred-chrome-bookmarks`二进制文件

## 配置步骤

### 1. 创建新的Workflow

1. 打开Alfred Preferences (`⌘,`)
2. 进入 **Workflows** 标签
3. 点击左下角的 `+` → **Blank Workflow**
4. 填写基本信息：
   - Name: `Chrome Bookmarks with Tags`
   - Description: `搜索和管理Chrome书签，支持多标签`
   - Bundle ID: `com.yourname.chrome-bookmarks`
   - Author: 你的名字

### 2. 配置搜索书签功能

#### 添加Script Filter

1. 右键workflow画布 → **Inputs** → **Script Filter**
2. 配置如下：

**基本设置:**
- Keyword: `cb`
- Title: `搜索Chrome书签`
- Placeholder Title: `输入关键词搜索书签...`
- Placeholder Subtext: `支持按标题、URL搜索`

**Script:**
```bash
/path/to/alfred-chrome-bookmarks search {query}
```

将 `/path/to/` 替换为你的实际二进制文件路径，例如：
```bash
/Users/yourname/alfred-chrome-bookmarks/target/release/alfred-chrome-bookmarks search {query}
```

**其他设置:**
- Language: `/bin/bash`
- 勾选 "Argument Required"
- 勾选 "Alfred filters results"

#### 添加Open URL动作

1. 右键workflow画布 → **Actions** → **Open URL**
2. 配置URL为: `{query}`
3. 将Script Filter的输出连接到Open URL动作

### 3. 配置列出标签功能 (可选)

#### 添加Script Filter

1. 右键workflow画布 → **Inputs** → **Script Filter**
2. 配置如下：

**基本设置:**
- Keyword: `cbtags`
- Title: `列出所有标签`
- Placeholder Title: `搜索标签...`

**Script:**
```bash
/path/to/alfred-chrome-bookmarks tags {query}
```

### 4. 配置刷新书签功能 (可选)

#### 添加Keyword Input

1. 右键workflow画布 → **Inputs** → **Keyword**
2. 配置如下：
   - Keyword: `cbrefresh`
   - Title: `刷新Chrome书签`

#### 添加Run Script

1. 右键workflow画布 → **Actions** → **Run Script**
2. Script:
```bash
/path/to/alfred-chrome-bookmarks refresh
```
3. 将Keyword输入连接到Run Script动作

## 高级配置

### 支持模糊搜索

修改搜索Script为：
```bash
/path/to/alfred-chrome-bookmarks search --fuzzy {query}
```

### 按标签过滤

创建一个新的Script Filter，keyword为 `cbt`:
```bash
# 用户输入格式: cbt work,dev rust
# 解析并执行
TAGS=$(echo "{query}" | awk '{print $1}')
QUERY=$(echo "{query}" | awk '{$1=""; print $0}' | xargs)
/path/to/alfred-chrome-bookmarks search --tags "$TAGS" "$QUERY"
```

### 添加热键 (Hotkey)

1. 右键workflow画布 → **Triggers** → **Hotkey**
2. 设置你喜欢的快捷键，例如 `⌘⇧B`
3. 连接到搜索Script Filter

## 使用示例

设置完成后，你可以：

1. **快速搜索**:
   - 按 `⌘Space` 打开Alfred
   - 输入 `cb github`
   - 选择书签并回车打开

2. **查看标签**:
   - 输入 `cbtags`
   - 浏览所有已有标签

3. **使用热键**:
   - 按设置的热键直接打开书签搜索

## 命令行添加标签

在终端中为书签添加标签：

```bash
# 在Chrome中打开你想要标记的页面
# 复制URL
# 在终端执行：
/path/to/alfred-chrome-bookmarks tag "复制的URL" tag1 tag2 tag3
```

## 建议的工作流程

1. 浏览网页时，将有用的页面加入Chrome书签
2. 定期使用命令行为书签添加标签
3. 在Alfred中使用 `cb` 快速搜索和访问书签
4. 使用 `cbtags` 查看和管理标签

## 故障排除

### 搜索无结果

- 确保Chrome书签文件存在
- 尝试使用 `cbrefresh` 刷新
- 检查二进制文件路径是否正确

### 权限问题

如果遇到权限问题：
```bash
chmod +x /path/to/alfred-chrome-bookmarks
```

### 调试

在Alfred Workflow编辑器中：
1. 右上角点击虫子图标打开调试
2. 执行搜索查看输出日志

## 环境变量

Workflow自动设置以下环境变量：
- `alfred_workflow_data`: 数据存储目录
- `alfred_workflow_cache`: 缓存目录

## 完整workflow截图示例

```
[Hotkey: ⌘⇧B]
       ↓
[Script Filter: cb]
       ↓
   [Open URL]
```

---

享受高效的书签管理体验！
