# 全局热键触发定位功能设计

## 背景

用户反馈：点击定位按钮会导致右键菜单消失，无法定位菜单窗口。需要全局热键直接触发定位功能，无需激活程序窗口。

## 需求

1. 默认快捷键 F8 触发定位模式
2. 界面显示当前快捷键，带修改图标（⚙）
3. 用户可点击修改图标更改快捷键
4. 重启程序后还原为默认 F8（不持久化）

## 设计方案

### 核心流程

1. 程序启动 → 注册全局热键 F8
2. 用户按 F8（任何情况下） → 程序进入定位模式
3. 定位完成后 → 退出定位模式，热键继续生效
4. 用户点击界面"⚙"图标 → 弹出输入框修改快捷键

### 改动范围

**修改文件**: `src/main.rs`

**新增内容**：
- 热键状态存储（`hotkey_key: String`，内存中）
- 热键注册/注销逻辑（`RegisterHotKey`/`UnregisterHotKey`）
- WM_HOTKEY 消息处理
- 快捷键显示和修改 UI

### 技术实现要点

**Windows API 热键注册**：
```rust
use windows::Win32::UI::WindowsAndMessaging::{
    RegisterHotKey, UnregisterHotKey, WM_HOTKEY,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{MOD_NOREPEAT, VK_F8};

// 注册热键
RegisterHotKey(hwnd, HOTKEY_ID, MOD_NOREPEAT, VK_F8);

// 注销热键
UnregisterHotKey(hwnd, HOTKEY_ID);
```

**WM_HOTKEY 消息处理**：
- eframe 使用 `winit` 事件循环，需要通过 `raw_window_handle` 获取 HWND
- 监听 WM_HOTKEY 消息（通过 `egui::Context::request_repaint` 检测）

**按键映射**：
- 支持按键格式：F1-F12、单个字母（A-Z）
- 输入格式：用户输入"F9"、"A"等字符串
- 内部映射到对应的 VK 码

### 界面改动

工具栏新增显示（位于定位按钮前）：
```
[快捷键: F8 ⚙]  [⊕ 定位]  [树形]  [刷新]  [重置]
```

点击 ⚙ 图标：
- 弹出 `egui::TextEdit` 单行输入框
- 用户输入新按键名称（如 F9、F7、A）
- 失去焦点或按 Enter 后生效

### 数据结构

```rust
struct MyApp {
    // ... 现有字段
    hotkey_key: String,       // 当前快捷键（如 "F8"）
    hotkey_id: u32,           // 热键 ID（用于注销）
    hotkey_editing: bool,     // 是否正在编辑快捷键
    hotkey_edit_text: String, // 编辑框内容
}
```

### 实现步骤

1. 新增 `MyApp` 字段：`hotkey_key`、`hotkey_id`、`hotkey_editing`、`hotkey_edit_text`
2. 在 `MyApp::new()` 中初始化默认值 F8，注册热键
3. 在 UI 工具栏添加快捷键显示和修改按钮
4. 处理修改逻辑：注销旧热键、注册新热键
5. 处理 WM_HOTKEY 消息：触发定位模式
6. 程序退出时注销热键

### 潜在问题

1. **热键冲突**：如果 F8 已被其他程序占用，注册会失败。界面提示"快捷键注册失败"。
2. **按键格式验证**：用户输入无效按键时，提示"无效快捷键"。
3. **程序退出**：需要确保热键注销，否则会残留。

### 测试要点

1. F8 触发定位：验证热键生效
2. 修改快捷键：修改为 F9，验证新热键生效
3. 热键冲突：占用 F8 后启动程序，验证提示
4. 重启还原：修改快捷键后重启，验证还原为 F8