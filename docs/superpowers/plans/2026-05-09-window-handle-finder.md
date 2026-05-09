# Window Handle Finder 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 创建一个类似Everything风格的Windows窗口查找工具，支持搜索、定位模式、树形视图，使用Rust + native-windows-gui。

**Architecture:** 分层架构：Windows API封装层 → 数据层 → UI层。使用native-windows-gui构建原生Windows界面，ListView显示窗口列表，延迟加载树形视图。

**Tech Stack:** Rust, native-windows-gui, clipboard-win, widestring

---

## 文件结构

```
src/
├── main.rs              # 主入口，启动应用、主窗口UI
├── window_info.rs       # WindowInfo数据结构定义
├── windows_api.rs       # Windows API封装（枚举窗口、获取信息）
├── clipboard.rs         # 剪贴板操作
└── ui/
    ├── mod.rs           # UI模块入口
    ├── app_state.rs     # 应用状态
    ├── tree_helper.rs   # 树形视图辅助函数
    └── locate_mode.rs   # 定位模式处理（鼠标捕获、窗口检测、光标样式）
Cargo.toml               # 项目配置和依赖
```

---

### Task 1: 项目初始化

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`

- [ ] **Step 1: 创建 Cargo.toml**

```toml
[package]
name = "window-handle-finder"
version = "0.1.0"
edition = "2021"

[dependencies]
native-windows-gui = "1.0"
native-windows-derive = "1.0"
clipboard-win = "5.0"
widestring = "1.0"
windows = { version = "0.58", features = [
    "Win32_Foundation",
    "Win32_UI_WindowsAndMessaging",
    "Win32_System_Threading",
] }

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

- [ ] **Step 2: 创建 src/main.rs 空壳**

```rust
fn main() {
    println!("Window Handle Finder - Starting...");
}
```

- [ ] **Step 3: 运行 cargo build 验证项目初始化成功**

Run: `cargo build`
Expected: 编译成功，无错误

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml Cargo.lock src/main.rs
git commit -m "feat: 初始化Rust项目结构"
```

---

### Task 2: 数据结构定义

**Files:**
- Create: `src/window_info.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: 创建 WindowInfo 数据结构**

```rust
// src/window_info.rs
use std::cell::RefCell;
use std::rc::Rc;

/// 窗口信息结构体
#[derive(Debug)]
pub struct WindowInfo {
    /// 窗口句柄
    pub hwnd: isize,
    /// 窗口标题
    pub title: String,
    /// 窗口类名
    pub class_name: String,
    /// 进程ID
    pub pid: u32,
    /// 进程名称（如 notepad.exe）
    pub process_name: String,
    /// 子窗口列表（树形模式，延迟加载）
    pub children: RefCell<Vec<Rc<WindowInfo>>>,
    /// 子窗口是否已加载
    pub children_loaded: RefCell<bool>,
}

impl WindowInfo {
    /// 创建新的窗口信息
    pub fn new(hwnd: isize, title: String, class_name: String, pid: u32, process_name: String) -> Self {
        Self {
            hwnd,
            title,
            class_name,
            pid,
            process_name,
            children: RefCell::new(Vec::new()),
            children_loaded: RefCell::new(false),
        }
    }

    /// 创建用于搜索的简化字符串
    pub fn search_text(&self) -> String {
        format!("{} {} {} {}", self.title, self.process_name, self.pid, self.class_name).to_lowercase()
    }
}
```

- [ ] **Step 2: 在 main.rs 中引入模块**

```rust
// src/main.rs
mod window_info;

fn main() {
    println!("Window Handle Finder - Starting...");
}
```

- [ ] **Step 3: 运行 cargo build 验证编译成功**

Run: `cargo build`
Expected: 编译成功

- [ ] **Step 4: Commit**

```bash
git add src/window_info.rs src/main.rs
git commit -m "feat: 定义WindowInfo数据结构"
```

---

### Task 3: Windows API封装 - 基础函数

**Files:**
- Create: `src/windows_api.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: 创建 Windows API 封装模块**

```rust
// src/windows_api.rs
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

/// Windows API 调用封装
pub struct WindowsApi;

impl WindowsApi {
    /// 获取窗口标题
    pub fn get_window_title(hwnd: isize) -> String {
        use windows::Win32::UI::WindowsAndMessaging::GetWindowTextW;
        use windows::Win32::Foundation::HWND;
        
        let mut buffer: [u16; 512] = [0; 512];
        let len = unsafe {
            GetWindowTextW(HWND(hwnd), &mut buffer)
        };
        if len == 0 {
            return String::new();
        }
        let os_string = OsString::from_wide(&buffer[..len as usize]);
        os_string.to_string_lossy().into_owned()
    }

    /// 获取窗口类名
    pub fn get_class_name(hwnd: isize) -> String {
        use windows::Win32::UI::WindowsAndMessaging::GetClassNameW;
        use windows::Win32::Foundation::HWND;
        
        let mut buffer: [u16; 256] = [0; 256];
        let len = unsafe {
            GetClassNameW(HWND(hwnd), &mut buffer)
        };
        if len == 0 {
            return String::new();
        }
        let os_string = OsString::from_wide(&buffer[..len as usize]);
        os_string.to_string_lossy().into_owned()
    }

    /// 获取窗口关联的进程ID
    pub fn get_window_pid(hwnd: isize) -> u32 {
        use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
        use windows::Win32::Foundation::HWND;
        
        unsafe {
            GetWindowThreadProcessId(HWND(hwnd), None)
        }
    }

    /// 根据PID获取进程名称
    pub fn get_process_name(pid: u32) -> String {
        use windows::Win32::Foundation::{CloseHandle, HANDLE};
        use windows::Win32::System::Threading::{
            OpenProcess, GetProcessImageFileNameW, PROCESS_QUERY_INFORMATION,
        };

        let process_handle = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION, false, pid) };
        if process_handle.is_invalid() {
            return "<无法访问>".to_string();
        }

        let mut buffer: [u16; 512] = [0; 512];
        let len = unsafe { GetProcessImageFileNameW(process_handle, &mut buffer) };

        unsafe { CloseHandle(process_handle) };

        if len == 0 {
            return "<无法访问>".to_string();
        }

        let os_string = OsString::from_wide(&buffer[..len as usize]);
        let full_path = os_string.to_string_lossy();
        // 提取文件名部分
        full_path
            .rsplit('\\')
            .next()
            .unwrap_or(&full_path)
            .to_string()
    }

    /// 判断窗口是否可见
    pub fn is_window_visible(hwnd: isize) -> bool {
        use windows::Win32::UI::WindowsAndMessaging::IsWindowVisible;
        use windows::Win32::Foundation::HWND;
        
        unsafe {
            IsWindowVisible(HWND(hwnd)).as_bool()
        }
    }
}
```

- [ ] **Step 2: 在 main.rs 中引入模块**

```rust
// src/main.rs
mod window_info;
mod windows_api;

fn main() {
    println!("Window Handle Finder - Starting...");
}
```

- [ ] **Step 3: 运行 cargo build 验证编译成功**

Run: `cargo build`
Expected: 编译成功

- [ ] **Step 4: Commit**

```bash
git add src/windows_api.rs src/main.rs
git commit -m "feat: Windows API基础函数封装"
```

---

### Task 4: Windows API封装 - 窗口枚举

**Files:**
- Modify: `src/windows_api.rs`

- [ ] **Step 1: 添加窗口枚举功能**

```rust
// 在 src/windows_api.rs 顶部添加
use crate::window_info::WindowInfo;
use std::rc::Rc;
use windows::Win32::Foundation::{HWND, LPARAM, BOOL};
use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, EnumChildWindows};

// 在 WindowsApi impl 中添加
impl WindowsApi {
    /// 遍历所有顶层窗口
    pub fn enum_windows() -> Vec<Rc<WindowInfo>> {
        let mut windows: Vec<Rc<WindowInfo>> = Vec::new();
        
        unsafe {
            EnumWindows(
                Some(enum_windows_callback),
                LPARAM(&mut windows as *mut Vec<Rc<WindowInfo>> as isize),
            );
        }
        
        windows
    }

    /// 遍历指定窗口的子窗口
    pub fn enum_child_windows(parent_hwnd: isize) -> Vec<Rc<WindowInfo>> {
        let mut children: Vec<Rc<WindowInfo>> = Vec::new();
        
        unsafe {
            EnumChildWindows(
                HWND(parent_hwnd),
                Some(enum_child_windows_callback),
                LPARAM(&mut children as *mut Vec<Rc<WindowInfo>> as isize),
            );
        }
        
        children
    }

    /// 创建 WindowInfo 对象
    pub fn create_window_info(hwnd: isize) -> Rc<WindowInfo> {
        Rc::new(WindowInfo::new(
            hwnd,
            Self::get_window_title(hwnd),
            Self::get_class_name(hwnd),
            Self::get_window_pid(hwnd),
            Self::get_process_name(Self::get_window_pid(hwnd)),
        ))
    }
}

/// EnumWindows 回调函数
extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let windows = unsafe { &mut *(lparam.0 as *mut Vec<Rc<WindowInfo>>) };
    let info = WindowsApi::create_window_info(hwnd.0);
    windows.push(info);
    BOOL(1) // 继续枚举
}

/// EnumChildWindows 回调函数
extern "system" fn enum_child_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let children = unsafe { &mut *(lparam.0 as *mut Vec<Rc<WindowInfo>>) };
    let info = WindowsApi::create_window_info(hwnd.0);
    children.push(info);
    BOOL(1) // 继续枚举
}
```

- [ ] **Step 2: 运行 cargo build 验证编译成功**

Run: `cargo build`
Expected: 编译成功

- [ ] **Step 3: Commit**

```bash
git add src/windows_api.rs
git commit -m "feat: 窗口枚举功能"
```

---

### Task 5: 剪贴板功能

**Files:**
- Create: `src/clipboard.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: 创建剪贴板模块**

```rust
// src/clipboard.rs
use clipboard_win::formats;

/// 剪贴板操作封装
pub struct ClipboardHelper;

impl ClipboardHelper {
    /// 复制文本到剪贴板
    pub fn copy_text(text: &str) -> bool {
        formats::Unicode::write_clipboard(text).is_ok()
    }

    /// 复制Handle到剪贴板（十六进制格式）
    pub fn copy_handle(hwnd: isize) -> bool {
        let handle_str = format!("0x{:08X}", hwnd);
        Self::copy_text(&handle_str)
    }

    /// 复制完整窗口信息到剪贴板
    pub fn copy_full_info(hwnd: isize, title: &str, class_name: &str, pid: u32, process_name: &str) -> bool {
        let info_str = format!(
            "Handle: 0x{:08X}\nTitle: {}\nClass: {}\nPID: {}\nProcess: {}",
            hwnd, title, class_name, pid, process_name
        );
        Self::copy_text(&info_str)
    }
}
```

- [ ] **Step 2: 在 main.rs 中引入模块**

```rust
// src/main.rs
mod window_info;
mod windows_api;
mod clipboard;

fn main() {
    println!("Window Handle Finder - Starting...");
}
```

- [ ] **Step 3: 运行 cargo build 验证编译成功**

Run: `cargo build`
Expected: 编译成功

- [ ] **Step 4: Commit**

```bash
git add src/clipboard.rs src/main.rs
git commit -m "feat: 剪贴板功能封装"
```

---

### Task 6: UI模块结构和应用状态

**Files:**
- Create: `src/ui/mod.rs`
- Create: `src/ui/app_state.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: 创建 UI 模块入口**

```rust
// src/ui/mod.rs
mod app_state;
pub mod locate_mode;
pub mod tree_helper;

pub use app_state::AppState;
pub use locate_mode::LocateMode;
pub use tree_helper::TreeHelper;
```

- [ ] **Step 2: 创建应用状态结构**

```rust
// src/ui/app_state.rs
use crate::window_info::WindowInfo;
use crate::windows_api::WindowsApi;
use std::rc::Rc;

/// 应用状态
pub struct AppState {
    /// 所有窗口列表
    pub all_windows: Vec<Rc<WindowInfo>>,
    /// 过滤后的窗口列表
    pub filtered_windows: Vec<Rc<WindowInfo>>,
    /// 当前搜索文本
    pub search_text: String,
    /// 是否为树形模式
    pub tree_mode: bool,
}

impl AppState {
    pub fn new() -> Self {
        let all_windows = WindowsApi::enum_windows();
        Self {
            all_windows: all_windows.clone(),
            filtered_windows: all_windows,
            search_text: String::new(),
            tree_mode: false,
        }
    }

    /// 刷新窗口列表
    pub fn refresh(&mut self) {
        self.all_windows = WindowsApi::enum_windows();
        self.apply_filter();
    }

    /// 应用搜索过滤
    pub fn apply_filter(&mut self) {
        if self.search_text.is_empty() {
            self.filtered_windows = self.all_windows.clone();
        } else {
            let search_lower = self.search_text.to_lowercase();
            self.filtered_windows = self.all_windows
                .iter()
                .filter(|w| w.search_text().contains(&search_lower))
                .cloned()
                .collect();
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **Step 3: 在 main.rs 中引入 UI 模块**

```rust
// src/main.rs
mod window_info;
mod windows_api;
mod clipboard;
mod ui;

fn main() {
    println!("Window Handle Finder - Starting...");
}
```

- [ ] **Step 4: 运行 cargo build 验证编译成功**

Run: `cargo build`
Expected: 编译成功

- [ ] **Step 5: Commit**

```bash
git add src/ui/mod.rs src/ui/app_state.rs src/main.rs
git commit -m "feat: UI模块结构和应用状态"
```

---

### Task 7: 定位模式模块（含鼠标光标）

**Files:**
- Create: `src/ui/locate_mode.rs`

- [ ] **Step 1: 创建定位模式模块（含鼠标光标和右键取消）**

```rust
// src/ui/locate_mode.rs
use crate::windows_api::WindowsApi;
use crate::window_info::WindowInfo;
use std::rc::Rc;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    SetCapture, ReleaseCapture, GetCursorPos, WindowFromPoint,
    SetCursor, LoadCursorW, IDC_CROSS,
};

/// 定位模式处理
pub struct LocateMode {
    active: bool,
}

impl LocateMode {
    pub fn new() -> Self {
        Self { active: false }
    }

    /// 开始定位模式
    pub fn start(&mut self, window_hwnd: isize) {
        self.active = true;
        unsafe {
            SetCapture(HWND(window_hwnd));
            // 设置十字准星光标
            let cursor = LoadCursorW(None, IDC_CROSS);
            SetCursor(cursor);
        }
    }

    /// 结束定位模式
    pub fn stop(&mut self) {
        self.active = false;
        unsafe {
            ReleaseCapture();
            // 恢复默认光标（传入None）
            SetCursor(None);
        }
    }

    /// 获取鼠标下的窗口信息
    pub fn get_window_at_cursor(&self) -> Option<Rc<WindowInfo>> {
        if !self.active {
            return None;
        }

        let mut point = windows::Win32::Foundation::POINT { x: 0, y: 0 };
        unsafe {
            GetCursorPos(&mut point);
        }

        let hwnd = unsafe { WindowFromPoint(point) };
        if hwnd.is_invalid() {
            return None;
        }

        Some(WindowsApi::create_window_info(hwnd.0))
    }

    pub fn is_active(&self) -> bool {
        self.active
    }
}

impl Default for LocateMode {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **Step 2: 运行 cargo build 验证编译成功**

Run: `cargo build`
Expected: 编译成功

- [ ] **Step 3: Commit**

```bash
git add src/ui/locate_mode.rs
git commit -m "feat: 定位模式模块（含十字光标）"
```

---

### Task 8: 树形视图辅助模块

**Files:**
- Create: `src/ui/tree_helper.rs`

- [ ] **Step 1: 创建树形视图辅助模块（含延迟加载）**

```rust
// src/ui/tree_helper.rs
use crate::window_info::WindowInfo;
use crate::windows_api::WindowsApi;
use native_windows_gui as nwg;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// 树形视图辅助函数
pub struct TreeHelper;

impl TreeHelper {
    /// 加载子窗口（延迟加载）
    pub fn load_children(parent: &Rc<WindowInfo>) {
        if *parent.children_loaded.borrow() {
            return;
        }
        
        let children = WindowsApi::enum_child_windows(parent.hwnd);
        *parent.children.borrow_mut() = children;
        *parent.children_loaded.borrow_mut() = true;
    }

    /// 将窗口列表填充到TreeView
    pub fn populate_tree(tree: &nwg::TreeView, windows: &[Rc<WindowInfo>]) {
        tree.clear();
        
        for win in windows {
            let text = if win.title.is_empty() {
                format!("<无标题> [{}:{} / {}]", win.process_name, win.pid, win.class_name)
            } else {
                format!("{} [{}:{} / {}]", win.title, win.process_name, win.pid, win.class_name)
            };
            
            let item = tree.insert_item(&text, None);
            
            // 插入一个占位子项，表示有子窗口可展开
            tree.insert_item("<点击展开...>", Some(&item));
        }
    }

    /// 处理节点展开事件（延迟加载子窗口）
    pub fn on_item_expand(
        tree: &nwg::TreeView, 
        item: &nwg::TreeViewItem, 
        window_map: &RefCell<HashMap<nwg::TreeViewItem, Rc<WindowInfo>>>
    ) {
        let map = window_map.borrow();
        if let Some(win) = map.get(item) {
            Self::load_children(win);
            
            // 获取当前节点的所有子项
            let children_items: Vec<nwg::TreeViewItem> = tree.children(item);
            // 移除所有占位子项
            for child_item in children_items {
                tree.remove_item(&child_item);
            }
            
            // 添加真实子窗口项
            let children = win.children.borrow();
            for child in children.iter() {
                let child_text = if child.title.is_empty() {
                    format!("<无标题> [{}]", child.class_name)
                } else {
                    format!("{} [{}]", child.title, child.class_name)
                };
                tree.insert_item(&child_text, Some(item));
            }
        }
    }
}
```

- [ ] **Step 2: 运行 cargo build 验证编译成功**

Run: `cargo build`
Expected: 编译成功

- [ ] **Step 3: Commit**

```bash
git add src/ui/tree_helper.rs
git commit -m "feat: 树形视图辅助模块（含延迟加载）"
```

---

### Task 9: 主窗口UI框架

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: 实现主窗口UI框架**

```rust
// src/main.rs
mod window_info;
mod windows_api;
mod clipboard;
mod ui;

use native_windows_gui as nwg;
use native_windows_derive as nwd;
use nwd::NwgUi;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::clipboard::ClipboardHelper;
use ui::{AppState, LocateMode, TreeHelper};
use crate::window_info::WindowInfo;

#[derive(Default, NwgUi)]
pub struct MainWindow {
    #[nwg_control(size: (500, 300), title: "Window Handle Finder")]
    #[nwg_events(OnInit: [init], OnWindowClose: [close])]
    window: nwg::Window,

    #[nwg_layout(parent: window, spacing: 4, margin: [4,4,4,4])]
    layout: nwg::FlexboxLayout,

    #[nwg_control(parent: window, focus: true)]
    #[nwg_layout_item(layout: layout, row: 0)]
    #[nwg_events(OnTextInput: [search_input])]
    search_box: nwg::TextInput,

    #[nwg_control(parent: window, text: "定位")]
    #[nwg_layout_item(layout: layout, row: 0)]
    #[nwg_events(OnButtonClick: [start_locate])]
    locate_btn: nwg::Button,

    #[nwg_control(parent: window, text: "树形")]
    #[nwg_layout_item(layout: layout, row: 0)]
    #[nwg_events(OnButtonClick: [toggle_tree_mode])]
    tree_btn: nwg::Button,

    #[nwg_control(parent: window, text: "刷新")]
    #[nwg_layout_item(layout: layout, row: 0)]
    #[nwg_events(OnButtonClick: [refresh_list])]
    refresh_btn: nwg::Button,

    #[nwg_control(parent: window, columns: &["标题", "进程名称", "PID", "Class Name"])]
    #[nwg_layout_item(layout: layout, row: 1, grow: 1)]
    #[nwg_events(OnListItemDoubleClick: [copy_handle], OnContextMenu: [show_context_menu])]
    list_view: nwg::ListView,

    #[nwg_control(parent: window)]
    #[nwg_layout_item(layout: layout, row: 1, grow: 1)]
    #[nwg_events(OnTreeItemExpand: [tree_item_expand])]
    tree_view: nwg::TreeView,

    #[nwg_control(parent: window)]
    context_menu: nwg::Menu,

    #[nwg_control(parent: context_menu, text: "复制Handle")]
    #[nwg_events(OnMenuItemSelected: [menu_copy_handle])]
    menu_copy_h: nwg::MenuItem,

    #[nwg_control(parent: context_menu, text: "复制Class Name")]
    #[nwg_events(OnMenuItemSelected: [menu_copy_class])]
    menu_copy_c: nwg::MenuItem,

    #[nwg_control(parent: context_menu, text: "复制完整信息")]
    #[nwg_events(OnMenuItemSelected: [menu_copy_full])]
    menu_copy_f: nwg::MenuItem,

    #[nwg_control(parent: context_menu, text: "刷新列表")]
    #[nwg_events(OnMenuItemSelected: [refresh_list])]
    menu_refresh: nwg::MenuItem,

    /// 应用状态
    data: RefCell<AppState>,
    /// 定位模式
    locate_mode: RefCell<LocateMode>,
    /// 树形视图节点到WindowInfo的映射
    tree_map: RefCell<HashMap<nwg::TreeViewItem, Rc<WindowInfo>>>,
}

impl MainWindow {
    fn init(&self) {
        // 初始化：默认显示列表视图
        self.tree_view.set_visible(false);
        
        let state = self.data.borrow();
        self.populate_list(&state.filtered_windows);
    }

    fn populate_list(&self, windows: &[Rc<WindowInfo>]) {
        self.list_view.clear();
        for win in windows {
            self.list_view.insert_item(&[
                if win.title.is_empty() { "<无标题>" } else { &win.title },
                &win.process_name,
                &win.pid.to_string(),
                &win.class_name,
            ]);
        }
    }

    fn search_input(&self) {
        let mut state = self.data.borrow_mut();
        state.search_text = self.search_box.text();
        state.apply_filter();
        let is_tree_mode = state.tree_mode;
        let filtered = state.filtered_windows.clone();
        drop(state); // 释放借用
        
        if is_tree_mode {
            TreeHelper::populate_tree(&self.tree_view, &filtered);
            // 更新tree_map
            let mut map = self.tree_map.borrow_mut();
            map.clear();
        } else {
            self.populate_list(&filtered);
        }
    }

    fn refresh_list(&self) {
        let mut state = self.data.borrow_mut();
        state.refresh();
        drop(state);
        
        let state = self.data.borrow();
        if state.tree_mode {
            TreeHelper::populate_tree(&self.tree_view, &state.filtered_windows);
        } else {
            self.populate_list(&state.filtered_windows);
        }
    }

    fn copy_handle(&self) {
        let idx = self.list_view.selected_item();
        if let Some(i) = idx {
            let state = self.data.borrow();
            if let Some(win) = state.filtered_windows.get(i) {
                ClipboardHelper::copy_handle(win.hwnd);
            }
        }
    }

    fn start_locate(&self) {
        self.locate_mode.borrow_mut().start(self.window.handle.hwnd().0);
        self.search_box.set_text("点击目标窗口...");
    }

    fn toggle_tree_mode(&self) {
        let mut state = self.data.borrow_mut();
        state.tree_mode = !state.tree_mode;
        
        if state.tree_mode {
            self.tree_btn.set_text("列表");
            self.list_view.set_visible(false);
            self.tree_view.set_visible(true);
            TreeHelper::populate_tree(&self.tree_view, &state.filtered_windows);
        } else {
            self.tree_btn.set_text("树形");
            self.tree_view.set_visible(false);
            self.list_view.set_visible(true);
            self.populate_list(&state.filtered_windows);
        }
    }

    fn tree_item_expand(&self, item: &nwg::TreeViewItem) {
        TreeHelper::on_item_expand(&self.tree_view, item, &self.tree_map);
    }

    fn show_context_menu(&self) {
        self.context_menu.popup(&self.window);
    }

    fn menu_copy_handle(&self) {
        self.copy_handle();
    }

    fn menu_copy_class(&self) {
        let idx = self.list_view.selected_item();
        if let Some(i) = idx {
            let state = self.data.borrow();
            if let Some(win) = state.filtered_windows.get(i) {
                ClipboardHelper::copy_text(&win.class_name);
            }
        }
    }

    fn menu_copy_full(&self) {
        let idx = self.list_view.selected_item();
        if let Some(i) = idx {
            let state = self.data.borrow();
            if let Some(win) = state.filtered_windows.get(i) {
                ClipboardHelper::copy_full_info(
                    win.hwnd, &win.title, &win.class_name, win.pid, &win.process_name
                );
            }
        }
    }

    fn close(&self) {
        nwg::stop_thread_dispatch();
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");

    let _app = MainWindow::build_ui(Default::default()).expect("Failed to build UI");

    nwg::dispatch_thread_events();
}
```

- [ ] **Step 2: 运行 cargo build 验证编译**

Run: `cargo build`
Expected: 编译成功

- [ ] **Step 3: 运行程序测试基本UI**

Run: `cargo run`
Expected: 窗口显示，列表有数据，列标题正确显示

- [ ] **Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: 主窗口UI框架和基本功能"
```

---

### Task 10: 定位模式和事件处理完善

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: 在 MainWindow 中添加鼠标和键盘事件处理**

在 `src/main.rs` 的 MainWindow 结构体的事件绑定部分添加：

```rust
// 在 window 的事件绑定中添加
#[nwg_events(
    OnInit: [init], 
    OnWindowClose: [close],
    OnMousePress: [handle_mouse_click],
    OnKeyPress: [handle_key_press]
)]
window: nwg::Window,
```

并在 impl MainWindow 中添加：

```rust
fn handle_mouse_click(&self, event: nwg::MousePressEvent) {
    // 右键取消定位模式（借用立即释放）
    if event.button == nwg::MouseButtons::Right 
        && self.locate_mode.borrow().is_active() {
        self.locate_mode.borrow_mut().stop();
        self.search_box.set_text("");
        return;
    }
    
    // 左键在定位模式下获取窗口
    if event.button == nwg::MouseButtons::Left 
        && self.locate_mode.borrow().is_active() {
        // 使用块限定借用作用域
        let win = {
            let mode = self.locate_mode.borrow();
            mode.get_window_at_cursor()
        }; // mode 在此释放
        
        if let Some(win) = win {
            // 将窗口添加到列表顶部
            let mut state = self.data.borrow_mut();
            state.all_windows.insert(0, win.clone());
            state.filtered_windows.insert(0, win.clone());
            drop(state);
            
            self.populate_list(&self.data.borrow().filtered_windows);
            ClipboardHelper::copy_handle(win.hwnd);
            self.search_box.set_text("");
        }
        self.locate_mode.borrow_mut().stop();
    }
}

fn handle_key_press(&self, event: nwg::KeyPressEvent) {
    // ESC取消定位模式（借用立即释放）
    if event.key_code == nwg::KeyCodes::ESCAPE 
        && self.locate_mode.borrow().is_active() {
        self.locate_mode.borrow_mut().stop();
        self.search_box.set_text("");
    }
}
```

- [ ] **Step 2: 运行 cargo build 验证编译**

Run: `cargo build`
Expected: 编译成功

- [ ] **Step 3: 运行程序测试定位模式**

Run: `cargo run`
测试：
1. 点击"定位"按钮，鼠标变成十字光标
2. 点击其他窗口，列表顶部添加该窗口
3. Handle自动复制到剪贴板
4. 按ESC取消定位模式
5. 右键点击取消定位模式

- [ ] **Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: 定位模式事件处理（含ESC和右键取消）"
```

---

### Task 11: 最终测试和优化

**Files:**
- All source files

- [ ] **Step 1: 运行完整功能测试**

Run: `cargo run`
测试项目：
1. 窗口列表显示是否正确，列标题是否正确
2. 搜索过滤是否正常（模糊匹配、不区分大小写）
3. 定位模式：十字光标、点击获取、ESC取消、右键取消
4. 树形视图切换、展开延迟加载
5. 双击复制Handle是否工作
6. 右键菜单是否显示（复制Handle、复制Class、复制完整信息、刷新）
7. 刷新按钮是否工作

- [ ] **Step 2: Release构建测试体积**

Run: `cargo build --release`
Expected: 体积 < 2MB

检查体积：
```bash
ls -lh target/release/window-handle-finder.exe
```

- [ ] **Step 3: 性能测试**

测试启动时间和搜索响应时间

- [ ] **Step 4: 修复发现的问题**

根据测试结果修复bug

- [ ] **Step 5: Commit 最终版本**

```bash
git add -A
git commit -m "feat: 最终版本完成"
```

---

## 完成标准

- [ ] 可执行文件体积 < 2MB
- [ ] 启动时间 < 100ms
- [ ] 搜索响应 < 50ms（千条数据）
- [ ] 所有核心功能正常工作
- [ ] Release构建成功