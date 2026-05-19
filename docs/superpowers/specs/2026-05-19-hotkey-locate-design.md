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

**全局状态（用于线程间通信）**：
```rust
use std::sync::atomic::{AtomicBool, Ordering};
static HOTKEY_TRIGGERED: AtomicBool = AtomicBool::new(false);
static ORIGINAL_WNDPROC: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
```

**获取 HWND 并子类化窗口**：
```rust
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowLongPtrW, GetWindowLongPtrW, GWLP_WNDPROC, WM_HOTKEY,
    CallWindowProcW, RegisterHotKey, UnregisterHotKey,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{MOD_NOREPEAT, VK_F8};
use windows::Win32::Foundation::{HWND, WPARAM, LPARAM, LRESULT};

fn setup_hotkey(ctx: &egui::Context) -> (Option<HWND>, String, u32) {
    let handle = match ctx.window_handle() {
        Ok(h) => h,
        Err(_) => return (None, "F8".to_string(), 0x77),
    };
    
    let raw = handle.as_raw();
    if let RawWindowHandle::Win32(win32) = raw {
        let hwnd = HWND(win32.hwnd as *mut c_void);
        
        // 保存原始窗口过程
        let original = unsafe { GetWindowLongPtrW(hwnd, GWLP_WNDPROC) };
        ORIGINAL_WNDPROC.store(original as *mut c_void, Ordering::SeqCst);
        
        // 子类化窗口
        unsafe {
            SetWindowLongPtrW(hwnd, GWLP_WNDPROC, hotkey_wndproc as usize as isize);
        }
        
        // 注册默认热键 F8
        let vk_f8 = 0x77;
        let result = unsafe { RegisterHotKey(hwnd, HOTKEY_ID, MOD_NOREPEAT, vk_f8) };
        if result.as_bool() {
            return (Some(hwnd), "F8".to_string(), vk_f8);
        }
        
        // F8 被占用，返回 None 表示失败，让用户手动修改
        return (Some(hwnd), "F8(失败)".to_string(), 0);
    }
    (None, "F8".to_string(), 0x77)
}
```

**自定义窗口过程**：
```rust
extern "system" fn hotkey_wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_HOTKEY {
        let hotkey_id = wparam.0 as u32;
        if hotkey_id == HOTKEY_ID {
            HOTKEY_TRIGGERED.store(true, Ordering::SeqCst);
            return LRESULT(0);
        }
    }
    
    // 调用原始窗口过程
    let original_ptr = ORIGINAL_WNDPROC.load(Ordering::SeqCst);
    unsafe {
        CallWindowProcW(
            Some(std::mem::transmute::<*mut c_void, isize>(original_ptr)),
            hwnd,
            msg,
            wparam,
            lparam,
        )
    }
}
```

**按键字符串到 VK 码映射**：
```rust
fn parse_hotkey(key: &str) -> Option<u32> {
    let upper = key.to_uppercase();
    // F1-F12: VK_F1 = 0x70, ..., VK_F12 = 0x7B
    if upper.starts_with('F') {
        let num: u32 = upper[1..].parse().ok()?;
        if num >= 1 && num <= 12 {
            return Some(0x70 + num - 1);
        }
    }
    // A-Z: 'A' = 0x41, ..., 'Z' = 0x5A
    if upper.len() == 1 {
        let c = upper.chars().next()?;
        if c >= 'A' && c <= 'Z' {
            return Some(c as u32);
        }
    }
    None
}

fn vk_to_string(vk: u32) -> String {
    // F1-F12
    if vk >= 0x70 && vk <= 0x7B {
        return format!("F{}", vk - 0x70 + 1);
    }
    // A-Z
    if vk >= 0x41 && vk <= 0x5A {
        return ((vk as u8) as char).to_string();
    }
    "F8".to_string() // 默认
}
```

**在 update() 中检测触发**：
```rust
// 每帧检测热键触发标志
if HOTKEY_TRIGGERED.load(Ordering::SeqCst) {
    HOTKEY_TRIGGERED.store(false, Ordering::SeqCst);
    if !self.locate_mode {
        self.locate_mode = true;
        self.locate_dragging = false;
        self.message = "按住鼠标左键或右键拖动到目标窗口，松开后定位".to_string();
    }
}
```

**修改快捷键逻辑**：
```rust
fn change_hotkey(&mut self, new_key: &str, hwnd: HWND) {
    let vk = match parse_hotkey(new_key) {
        Some(v) => v,
        None => {
            self.message = "无效快捷键（支持 F1-F12 或 A-Z）".to_string();
            return;
        }
    };
    
    // 注销旧热键
    unsafe { UnregisterHotKey(hwnd, HOTKEY_ID) };
    
    // 注册新热键
    let success = unsafe { RegisterHotKey(hwnd, HOTKEY_ID, MOD_NOREPEAT, vk).as_bool() };
    
    if success {
        self.hotkey_key = vk_to_string(vk);
        self.hotkey_vk = vk;
        self.message.clear();
    } else {
        self.message = format!("快捷键 {} 注册失败（已被占用）", new_key);
        // 尝试恢复之前的热键
        unsafe { RegisterHotKey(hwnd, HOTKEY_ID, MOD_NOREPEAT, self.hotkey_vk) };
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
        let resp = ui.add(
            egui::TextEdit::singleline(&mut self.hotkey_edit_text)
                .desired_width(40.0)
                .hint_text("F1-F12/A-Z")
        );
        // Enter 确认修改
        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if let Some(hwnd) = self.hwnd {
                self.change_hotkey(&self.hotkey_edit_text, hwnd);
            }
            self.hotkey_editing = false;
        }
        // ESC 取消编辑
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.hotkey_editing = false;
            self.message.clear();
        }
        // 失焦取消编辑（不保存）
        if resp.lost_focus() && !ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            self.hotkey_editing = false;
        }
    } else {
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

**初始化**：
```rust
fn new(cc: &eframe::CreationContext) -> Self {
    let (hwnd, hotkey_key, hotkey_vk) = setup_hotkey(&cc.egui_ctx);
    let mut message = String::new();
    if hotkey_key.contains("(失败)") {
        message = "F8 已被占用，请点击⚙修改快捷键".to_string();
    }
    Self {
        // ... 现有字段
        hotkey_key,
        hotkey_vk,
        hotkey_editing: false,
        hotkey_edit_text: String::new(),
        hwnd,
        message,
    }
}
```

### 实现步骤

1. 新增 `MyApp` 字段
2. 新增常量 `HOTKEY_ID`
3. 新增全局原子变量 `HOTKEY_TRIGGERED`、`ORIGINAL_WNDPROC`
4. 新增函数 `parse_hotkey`、`vk_to_string`、`hotkey_wndproc`、`setup_hotkey`、`change_hotkey`
5. 在 `MyApp::new()` 中调用 `setup_hotkey`
6. 在 `update()` 中检测热键触发、添加 UI
7. 程序退出时自动清理（窗口销毁时热键自动注销）

### 潜在问题

1. **热键冲突**：注册失败时界面提示，并尝试恢复之前的热键
2. **无效输入**：输入非 F1-F12/A-Z 时提示"无效快捷键"
3. **程序退出**：窗口销毁时热键自动注销，无需手动清理
4. **最小化状态**：热键仍然生效，因为 RegisterHotKey 是全局的

### 测试要点

1. F8 触发定位：验证热键生效（程序在后台时也能触发）
2. 修改快捷键：修改为 F9，验证新热键生效
3. 无效输入：输入"abc"，验证提示"无效快捷键（支持 F1-F12 或 A-Z）"
4. ESC 取消：编辑框中输入后按 ESC，验证取消不保存
5. 失焦取消：编辑框中输入后点击别处，验证取消不保存
6. 热键冲突：占用 F8 后启动程序，验证提示"F8 已被占用，请点击⚙修改"
7. 重启还原：修改快捷键后重启，验证还原为 F8
8. 程序最小化：验证热键仍然生效