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

1. 程序启动 → 获取 HWND → 子类化窗口过程 → 注册全局热键 F8
2. 用户按 F8（任何情况下） → WM_HOTKEY 消息 → 触发定位模式
3. 定位完成后 → 退出定位模式，热键继续生效
4. 用户点击界面"⚙"图标 → 内联编辑框修改快捷键

### 改动范围

**修改文件**: `src/main.rs`

**新增内容**：
- 热键状态存储（内存中）
- 窗口子类化（拦截 WM_HOTKEY）
- 热键注册/注销
- 按键映射函数
- 快捷键显示和修改 UI

### 技术实现要点

**常量定义**：
```rust
const HOTKEY_ID: u32 = 0x0001;
```

**获取 HWND 并子类化窗口**：
```rust
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowLongPtrW, GWLP_WNDPROC, WM_HOTKEY,
};

// 在 MyApp::new() 中，通过 CreationContext 获取 HWND
fn setup_hotkey(cc: &eframe::CreationContext) {
    let handle = cc.egui_ctx.window_handle().unwrap();
    let raw = handle.as_raw();
    if let RawWindowHandle::Win32(win32) = raw {
        let hwnd = HWND(win32.hwnd as *mut c_void);
        // 子类化窗口过程，保存原始 WNDPROC
        let original_wndproc = unsafe {
            SetWindowLongPtrW(hwnd, GWLP_WNDPROC, Some(hotkey_wndproc) as *mut c_void)
        };
        // 注册热键
        RegisterHotKey(hwnd, HOTKEY_ID, MOD_NOREPEAT, VK_F8);
    }
}
```

**自定义窗口过程（拦截 WM_HOTKEY）**：
```rust
static mut HOTKEY_TRIGGERED: bool = false;
static mut ORIGINAL_WNDPROC: isize = 0;

extern "system" fn hotkey_wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_HOTKEY && wparam.0 == HOTKEY_ID as usize {
        unsafe { HOTKEY_TRIGGERED = true; }
        return LRESULT(0);
    }
    // 调用原始窗口过程
    unsafe {
        CallWindowProcW(HWND(ORIGINAL_WNDPROC as *mut c_void), hwnd, msg, wparam, lparam)
    }
}
```

**在 update() 中检测触发**：
```rust
// 每帧检测热键触发标志
if unsafe { HOTKEY_TRIGGERED } {
    unsafe { HOTKEY_TRIGGERED = false; }
    self.locate_mode = true;
    self.message = "按住鼠标左键或右键拖动到目标窗口，松开后定位".to_string();
}
```

**按键字符串到 VK 码映射**：
```rust
fn parse_hotkey(key: &str) -> Option<u32> {
    match key.to_uppercase().as_str() {
        "F1" => Some(0x70), "F2" => Some(0x71), ..., "F12" => Some(0x7B),
        "A" => Some(0x41), ..., "Z" => Some(0x5A),
        _ => None,
    }
}
```

**修改快捷键逻辑**：
```rust
fn change_hotkey(&mut self, new_key: &str, hwnd: HWND) {
    // 注销旧热键
    UnregisterHotKey(hwnd, HOTKEY_ID);
    
    // 解析新按键
    let vk = parse_hotkey(new_key);
    if let Some(vk) = vk {
        if RegisterHotKey(hwnd, HOTKEY_ID, MOD_NOREPEAT, vk).as_bool() {
            self.hotkey_key = new_key.to_uppercase();
            self.message.clear();
        } else {
            self.message = "快捷键注册失败（可能已被占用）".to_string();
            // 恢复默认
            RegisterHotKey(hwnd, HOTKEY_ID, MOD_NOREPEAT, VK_F8);
        }
    } else {
        self.message = "无效快捷键（支持 F1-F12 或 A-Z）".to_string();
    }
}
```

### 界面改动

工具栏新增显示（位于定位按钮前）：
```
[快捷键: F8 ⚙]  [⊕ 定位]  [树形]  [刷新]  [重置]
```

**UI 实现（内联编辑）**：
```rust
ui.horizontal(|ui| {
    ui.label("快捷键:");
    if self.hotkey_editing {
        // 显示编辑框
        let resp = ui.add(egui::TextEdit::singleline(&mut self.hotkey_edit_text).desired_width(40.0));
        if resp.lost_focus() {
            // 失去焦点时应用更改
            self.change_hotkey(&self.hotkey_edit_text, hwnd);
            self.hotkey_editing = false;
        }
    } else {
        // 显示当前快捷键 + 修改按钮
        ui.label(&self.hotkey_key);
        if ui.button("⚙").clicked() {
            self.hotkey_editing = true;
            self.hotkey_edit_text = self.hotkey_key.clone();
        }
    }
});
```

### 数据结构

```rust
struct MyApp {
    // ... 现有字段
    hotkey_key: String,       // 当前快捷键显示文本（如 "F8"）
    hotkey_vk: u32,           // 当前快捷键 VK 码（如 0x77）
    hotkey_editing: bool,     // 是否正在编辑
    hotkey_edit_text: String, // 编辑框内容
    hwnd: Option<HWND>,       // 窗口句柄（用于热键操作）
}
```

### 实现步骤

1. 新增 `MyApp` 字段
2. 新增常量 `HOTKEY_ID`
3. 新增全局变量 `HOTKEY_TRIGGERED`、`ORIGINAL_WNDPROC`
4. 新增函数 `parse_hotkey`、`hotkey_wndproc`
5. 在 `MyApp::new()` 中：获取 HWND、子类化窗口、注册热键
6. 在 `update()` 中：检测热键触发、添加 UI
7. 新增方法 `change_hotkey`

### 潜在问题

1. **热键冲突**：注册失败时界面提示"快捷键注册失败"，并恢复默认 F8
2. **无效输入**：输入非 F1-F12/A-Z 时提示"无效快捷键"
3. **程序退出**：eframe 正常退出时会自动清理窗口，热键随之注销

### 测试要点

1. F8 触发定位：验证热键生效（程序在后台时也能触发）
2. 修改快捷键：修改为 F9，验证新热键生效
3. 无效输入：输入"abc"，验证提示
4. 重启还原：修改快捷键后重启，验证还原为 F8