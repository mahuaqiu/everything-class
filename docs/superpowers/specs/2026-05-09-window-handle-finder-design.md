# Window Handle Finder - 设计文档

## 项目概述

一个类似Everything风格的Windows窗口查找工具，使用Rust开发，用于快速定位窗口Handle、Class Name、标题、PID和进程名称信息。

## 目标用户

自动化测试和UI自动化工具开发者，需要精确获取窗口Handle来操作控件。

## 核心功能

### 1. 实时搜索
- 支持搜索维度：PID、进程名称、标题、Class Name
- 匹配规则：模糊匹配，不区分大小写
- 输入即刻过滤响应
- 支持按进程名称列出该进程所有窗口

### 2. 定位模式
- 点击"定位"按钮进入定位模式
- 鼠标变成瞄准状态
- 点击目标窗口获取信息
- 无悬停预览，点击后才显示信息
- **取消方式**：按ESC键或右键点击取消定位模式

### 3. 显示模式切换
- **列表模式（默认）**：显示顶层窗口，四列信息
- **树形模式**：显示窗口层级结构，可展开/折叠子窗口
- 树形模式采用延迟加载策略：展开节点时才遍历子窗口

### 4. 交互功能
- **双击列表项**：复制Handle到剪贴板（用户最需要的值）
- **右键菜单**：
  - 复制Handle
  - 复制Class Name
  - 复制完整信息
  - 刷新窗口列表
- **刷新按钮**：手动刷新窗口列表（可选定时自动刷新）

## 界面设计

### 列表模式
```
[ 搜索框 ] [定位] [树形/列表] [刷新]
────────────────────────────────────────
标题          进程名称    PID   Class Name
────────────────────────────────────────
无标题-记事本  notepad.exe 1234  Notepad
Google Chrome chrome.exe  5678  Chrome_WidgetWin_1
```

### 树形模式
```
▼ 无标题 - 记事本 [notepad.exe:1234 / Notepad]
  ├─ 编辑区 [Edit]
  └─ 状态栏 [msctls_statusbar32]
▼ Google Chrome [chrome.exe:5678 / Chrome_WidgetWin_1]
  └─ 内容区域 [Chrome_RenderWidgetHost]
```

### 界面要求
- 窗口尺寸紧凑，能显示信息即可
- 无详情面板（列表已包含全部所需信息）
- 原生Windows风格（类似Everything）

## 技术选型

### 语言与框架
- **语言**：Rust（高性能、内存安全）
- **GUI框架**：native-windows-gui（原生Windows控件）
- **体积目标**：< 2MB
- **启动目标**：< 100ms

### Windows API
| API | 用途 |
|-----|------|
| EnumWindows | 遍历顶层窗口 |
| EnumChildWindows | 遍历子窗口 |
| GetClassNameW | 获取窗口类名（Unicode版）|
| GetWindowTextW | 获取窗口标题（Unicode版）|
| GetWindowThreadProcessId | 获取窗口关联的PID |
| OpenProcess | 打开进程获取句柄 |
| GetProcessImageFileNameW | 根据PID获取进程名称 |
| CloseHandle | 关闭进程句柄 |
| SetCapture / ReleaseCapture | 定位模式鼠标捕获 |
| GetCursorPos | 获取鼠标位置 |
| WindowFromPoint | 获取鼠标下的窗口 |
| IsWindowVisible | 判断窗口可见性 |

### 剪贴板操作
使用 `clipboard-win` crate 处理剪贴板操作，避免直接调用Windows剪贴板API的复杂性。

### 字符串编码
- Windows API使用UTF-16（wide string）
- Rust内部使用UTF-8
- 使用 `widestring` crate 进行转换
- 使用 `encode_utf16()` / `decode_utf16()` 处理

## 数据结构

```rust
struct WindowInfo {
    hwnd: HWND,           // 窗口句柄（内部使用，双击复制）
    title: String,        // 窗口标题
    class_name: String,   // 窗口类名
    pid: u32,             // 进程ID（仅顶层窗口存储）
    process_name: String, // 进程名称（仅顶层窗口存储，子窗口可复用父窗口信息）
    children: Vec<WindowInfo>, // 子窗口（树形模式，延迟加载）
    children_loaded: bool,     // 子窗口是否已加载
}
// 注：子窗口的pid和process_name通常与父窗口相同，可在显示时复用父窗口数据以减少冗余
```

## 边界情况处理

| 边界情况 | 处理方式 |
|----------|----------|
| 无标题窗口 | 显示为空字符串 |
| 隐藏窗口 | 默认包含，无特殊处理 |
| 权限不足的进程 | 进程名称显示为 `<无法访问>` |
| 进程已退出 | 正常显示已缓存的信息 |
| Class Name过长 | 自动截断显示，完整值可复制 |
| UWP窗口 | GetWindowText可能失败，显示空字符串 |

## 性能指标

- 体积 < 2MB
- 启动时间 < 100ms
- 搜索响应 < 50ms（千条数据）
- 使用Windows原生ListView虚拟化处理大量数据
- 树形模式延迟加载避免启动时遍历所有子窗口

## 非功能需求

- 无Handle列显示（双击复制即可获取）
- 列顺序：标题、进程名称、PID、Class Name
- 极简界面，无多余功能
- 单EXE文件，无外部依赖
- 错误处理：所有Windows API调用检查返回值，失败时提供合理默认值

## 参考项目

- accessibility-insights-windows：Windows API调用方式参考
- Everything：界面风格和响应速度参考
- Spy++：定位模式和树形结构参考

---
创建日期：2026-05-09