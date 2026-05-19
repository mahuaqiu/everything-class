# 命令行窗口查询功能设计

## 概述

为 Window Class Finder 添加命令行查询模式，支持通过 `--class` 参数查询指定类名的窗口信息。

## 需求

- 命令行优先模式：有 `--class` 参数时执行查询并退出，不启动 GUI；无参数时正常启动 GUI
- 输出格式：JSON
- 匹配方式：精确匹配 class_name
- 不增加新依赖，保持可执行文件体积 < 2MB

## 使用示例

```bash
# 查询窗口
window-class-finder.exe --class="Notepad"

# 输出（存在）
{"hwnd": 12345, "title": "无标题 - 记事本", "class_name": "Notepad", "pid": 1234, "process_name": "notepad.exe"}

# 输出（不存在）
null

# 无参数启动 GUI
window-class-finder.exe
```

## 参数格式

支持两种形式：
- `--class="xxxx"` - 等号形式
- `--class xxxx` - 空格形式

## 实现

### 修改文件

`src/main.rs` - 添加参数解析和查询逻辑

### 实现要点

1. 在 `main()` 函数开头解析 `std::env::args()`
2. 检测 `--class` 参数，提取类名
3. 调用现有 `WindowsApi::enum_windows()` 获取窗口列表
4. 精确匹配 `class_name`
5. 手动构建 JSON 输出（使用字符串格式化，处理转义）
6. 执行查询后立即 `return`，不进入 GUI 代码

### JSON 输出格式

```json
{
  "hwnd": 12345,
  "title": "窗口标题",
  "class_name": "xxxx",
  "pid": 1234,
  "process_name": "app.exe"
}
```

字段说明：
- `hwnd`: 窗口句柄（整数）
- `title`: 窗口标题（字符串，空标题输出空字符串）
- `class_name`: 窗口类名（字符串）
- `pid`: 进程 ID（整数）
- `process_name`: 进程名称（字符串）

### 字符串转义

JSON 字符串需转义特殊字符：
- `"` → `\"`
- `\` → `\\`
- `\n` → `\\n`
- `\r` → `\\r`
- `\t` → `\\t`

### 代码结构

```rust
fn main() {
    // 参数解析
    if let Some(class_name) = parse_class_arg() {
        query_and_output(&class_name);
        return;
    }
    
    // 现有 GUI 代码...
}

fn parse_class_arg() -> Option<String> {
    // 解析 --class="xxxx" 或 --class xxxx
}

fn query_and_output(class_name: &str) {
    // 查询并输出 JSON
}

fn format_json(win: &WindowInfo) -> String {
    // 手动构建 JSON
}

fn escape_json(s: &str) -> String {
    // 转义特殊字符
}
```

## 测试验证

1. 基本功能测试：
   - `--class="Notepad"` 查询记事本窗口
   - 查询不存在的类名，输出 `null`

2. 参数形式测试：
   - `--class="Edit"` 和 `--class Edit` 都能正常工作

3. GUI 启动测试：
   - 无参数时正常启动 GUI

4. 特殊字符测试：
   - 窗口标题包含引号、反斜杠时 JSON 输出正确