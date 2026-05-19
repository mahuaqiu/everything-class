# 全局热键触发定位功能实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 添加全局热键 F8 触发定位模式，解决点击按钮导致右键菜单消失的问题。

**Architecture:** 使用 Windows API RegisterHotKey + 窗口子类化拦截 WM_HOTKEY 消息，在 egui update() 中检测触发标志。

**Tech Stack:** Rust, eframe/egui, Windows API (RegisterHotKey, SetWindowLongPtrW), raw-window-handle

---

## 文件结构

**修改文件:**
- `Cargo.toml` - 添加 raw-window-handle 依赖
- `src/main.rs` - 热键逻辑和 UI

---

### Task 1: 添加依赖

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: 添加 raw-window-handle 依赖**

在 `Cargo.toml` 的 `[dependencies]` 中添加：
```toml
raw-window-handle = "0.6"
```

- [ ] **Step 2: 检查依赖**

Run: `cargo check --offline 2>&1 || echo "网络问题，跳过"`
Expected: 如果有缓存则成功，否则忽略网络错误

---

### Task 2: 新增常量和全局变量

**Files:**
- Modify: `src/main.rs:1-10` (文件顶部)

- [ ] **Step 1: 在文件顶部添加常量和全局变量**

在 `mod window_info;` 之后添加：
```rust
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

const HOTKEY_ID: u32 = 0x0001;
static HOTKEY_TRIGGERED: AtomicBool = AtomicBool::new(false);
static ORIGINAL_WNDPROC: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
```

- [ ] **Step 2: 编译检查**

Run: `cargo check --offline 2>&1 || echo "跳过"`
Expected: 检查语法是否正确

---

### Task 3: 新增辅助函数

**Files:**
- Modify: `src/main.rs` (新增函数)

- [ ] **Step 1: 添加按键映射函数**

在 `MyApp` 结构体之前添加：
```rust
/// 解析快捷键字符串为 VK 码
fn parse_hotkey(key: &str) -> Option<u32> {
    let upper = key.to_uppercase();
    // F1-F12: VK_F1 = 0x70, ..., VK_F12 = 0x7B
    if upper.starts_with('F') {
        if upper.len() > 1 {
            let num: u32 = upper[1..].parse().ok()?;
            if num >= 1 && num <= 12 {
                return Some(0x70 + num - 1);
            }
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

/// VK 码转字符串
fn vk_to_string(vk: u32) -> String {
    // F1-F12
    if vk >= 0x70 && vk <= 0x7B {
        return format!("F{}", vk - 0x70 + 1);
    }
    // A-Z
    if vk >= 0x41 && vk <= 0x5A {
        return ((vk as u8) as char).to_string();
    }
    "F8".to_string()
}
```

- [ ] **Step 2: 编译检查**

Run: `cargo check --offline 2>&1 || echo "跳过"`

---

### Task 4: 新增窗口子类化函数

**Files:**
- Modify: `src/main.rs` (新增函数)

- [ ] **Step 1: 添加窗口过程和 setup 函数**

在辅助函数之后添加：
```rust
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowLongPtrW, GetWindowLongPtrW, GWLP_WNDPROC, WM_HOTKEY,
    CallWindowProcW, RegisterHotKey, UnregisterHotKey,
};
use windows::Win32::UI::Input::KeyboardAndMouse::MOD_NOREPEAT;
use windows::Win32::Foundation::{HWND, WPARAM, LPARAM, LRESULT};

/// 自定义窗口过程（拦截 WM_HOTKEY）
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

/// 设置热键（子类化窗口 + 注册热键）
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
        
        // F8 被占用
        return (Some(hwnd), "F8(失败)".to_string(), 0);
    }
    (None, "F8".to_string(), 0x77)
}
```

- [ ] **Step 2: 编译检查**

Run: `cargo check --offline 2>&1 || echo "跳过"`

---

### Task 5: 修改 MyApp 结构体

**Files:**
- Modify: `src/main.rs:80-105` (MyApp 结构体)

- [ ] **Step 1: 添加新字段到 MyApp**

在现有字段后添加：
```rust
struct MyApp {
    windows: Vec<WindowInfo>,
    filtered: Vec<WindowInfo>,
    search: String,
    tree_mode: bool,
    locate_mode: bool,
    expanded: Vec<bool>,
    message: String,
    locate_dragging: bool,
    // 新增热键相关字段
    hotkey_key: String,
    hotkey_vk: u32,
    hotkey_editing: bool,
    hotkey_edit_text: String,
    hwnd: Option<HWND>,
}
```

- [ ] **Step 2: 修改 new() 初始化**

修改 `MyApp::new()` 函数：
```rust
fn new(cc: &eframe::CreationContext) -> Self {
    let (hwnd, hotkey_key, hotkey_vk) = setup_hotkey(&cc.egui_ctx);
    let mut message = String::new();
    if hotkey_key.contains("(失败)") {
        message = "F8 已被占用，请点击⚙修改快捷键".to_string();
    }
    
    let windows = WindowsApi::enum_windows();
    let len = windows.len();
    Self {
        windows: windows.clone(),
        filtered: windows,
        search: String::new(),
        tree_mode: false,
        locate_mode: false,
        expanded: vec![false; len],
        message,
        locate_dragging: false,
        hotkey_key,
        hotkey_vk,
        hotkey_editing: false,
        hotkey_edit_text: String::new(),
        hwnd,
    }
}
```

- [ ] **Step 3: 编译检查**

Run: `cargo check --offline 2>&1 || echo "跳过"`

---

### Task 6: 添加 change_hotkey 方法

**Files:**
- Modify: `src/main.rs` (MyApp impl)

- [ ] **Step 1: 在 MyApp impl 中添加方法**

在 `copy_class` 方法后添加：
```rust
fn change_hotkey(&mut self, new_key: &str) {
    if let Some(hwnd) = self.hwnd {
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
            if self.hotkey_vk > 0 {
                unsafe { RegisterHotKey(hwnd, HOTKEY_ID, MOD_NOREPEAT, self.hotkey_vk) };
            }
        }
    }
}
```

- [ ] **Step 2: 编译检查**

Run: `cargo check --offline 2>&1 || echo "跳过"`

---

### Task 7: 修改 update() 添加热键检测和 UI

**Files:**
- Modify: `src/main.rs:134-180` (update 函数)

- [ ] **Step 1: 在 update() 开头添加热键触发检测**

在 `egui::CentralPanel::default().show(ctx, |ui| {` 之后，工具栏之前添加：
```rust
// 检测热键触发
if HOTKEY_TRIGGERED.load(Ordering::SeqCst) {
    HOTKEY_TRIGGERED.store(false, Ordering::SeqCst);
    if !self.locate_mode {
        self.locate_mode = true;
        self.locate_dragging = false;
        self.message = "按住鼠标左键或右键拖动到目标窗口，松开后定位".to_string();
    }
}
```

- [ ] **Step 2: 在工具栏添加快捷键 UI**

在搜索框之后、定位按钮之前添加：
```rust
// 快捷键显示和编辑
ui.horizontal(|ui| {
    ui.label("快捷键:");
    if self.hotkey_editing {
        let resp = ui.add(
            egui::TextEdit::singleline(&mut self.hotkey_edit_text)
                .desired_width(40.0)
                .hint_text("F1-F12/A-Z")
        );
        // Enter 确认
        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            self.change_hotkey(&self.hotkey_edit_text);
            self.hotkey_editing = false;
        }
        // ESC 取消
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.hotkey_editing = false;
            self.message.clear();
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

- [ ] **Step 3: 编译检查**

Run: `cargo check --offline 2>&1 || echo "跳过"`

---

### Task 8: 手动测试验证

**Files:**
- None (手动测试)

- [ ] **Step 1: 运行程序**

Run: `cargo run`
Expected: 程序启动，显示"快捷键: F8 ⚙"

- [ ] **Step 2: 测试 F8 触发定位**

操作：按 F8（程序在后台时）
Expected: 进入定位模式，十字光标出现

- [ ] **Step 3: 测试修改快捷键**

操作：点击⚙图标，输入"F9"，按 Enter
Expected: 快捷键改为 F9，按 F9 触发定位

- [ ] **Step 4: 测试无效输入**

操作：点击⚙图标，输入"abc"，按 Enter
Expected: 显示"无效快捷键（支持 F1-F12 或 A-Z）"

- [ ] **Step 5: 测试 ESC 取消**

操作：点击⚙图标，输入"F7"，按 ESC
Expected: 取消编辑，快捷键仍为原值

---

### Task 9: 提交代码

**Files:**
- `Cargo.toml`
- `src/main.rs`

- [ ] **Step 1: 提交改动**

```bash
git add Cargo.toml src/main.rs
git commit -m "feat: 添加全局热键 F8 触发定位功能"
```

Expected: 提交成功

- [ ] **Step 2: 查看提交历史**

Run: `git log --oneline -3`
Expected: 显示最新提交