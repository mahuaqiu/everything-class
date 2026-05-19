// 移除静态隐藏控制台，改为动态控制
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod window_info;
mod windows_api;

use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

const HOTKEY_ID: u32 = 0x0001;
static HOTKEY_TRIGGERED: AtomicBool = AtomicBool::new(false);
static ORIGINAL_WNDPROC: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

use eframe::egui;
use window_info::WindowInfo;
use windows_api::WindowsApi;
use arboard::Clipboard;

/// 加载应用图标
fn load_icon() -> egui::IconData {
    // 内嵌的 PNG 图标数据
    const ICON_DATA: &[u8] = include_bytes!("../assets/app.png");

    if let Ok(image) = image::load_from_memory(ICON_DATA) {
        let pixels = image.to_rgba8();
        egui::IconData {
            width: pixels.width() as u32,
            height: pixels.height() as u32,
            rgba: pixels.into_raw(),
        }
    } else {
        egui::IconData::default()
    }
}

fn main() {
    // 命令行参数解析：有 --class 参数时执行查询并退出
    if let Some(class_name) = parse_class_arg() {
        query_and_output(&class_name);
        return;
    }

    // 无参数，启动 GUI
    // 释放控制台，避免显示黑窗口
    #[link(name = "kernel32")]
    extern "system" {
        fn FreeConsole() -> i32;
    }
    unsafe { FreeConsole() };

    // 加载图标
    let icon = load_icon();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1050.0, 600.0])
            .with_title("Window Class Finder")
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        "Window Class Finder",
        options,
        Box::new(|cc| {
            // 设置中文字体
            setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(MyApp::new(cc)))
        }),
    ).expect("Failed to start");
}

/// 设置支持中文的自定义字体
fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // 运行时加载系统字体（微软雅黑）
    if let Ok(font_data) = std::fs::read("C:\\Windows\\Fonts\\msyh.ttc") {
        fonts.font_data.insert(
            "my_font".to_owned(),
            egui::FontData::from_owned(font_data),
        );

        // 设置为首选字体
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "my_font".to_owned());

        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "my_font".to_owned());

        ctx.set_fonts(fonts);
    }
}

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

// 使用 FFI 直接定义热键相关函数
#[link(name = "user32")]
extern "system" {
    fn RegisterHotKey(hWnd: isize, id: i32, fsModifiers: u32, vk: u32) -> i32;
    fn UnregisterHotKey(hWnd: isize, id: i32) -> i32;
    fn SetWindowLongPtrW(hWnd: isize, nIndex: i32, dwNewLong: isize) -> isize;
    fn GetWindowLongPtrW(hWnd: isize, nIndex: i32) -> isize;
    fn CallWindowProcW(lpPrevWndFunc: isize, hWnd: isize, Msg: u32, wParam: usize, lParam: isize) -> isize;
}

const GWLP_WNDPROC: i32 = -4;
const WM_HOTKEY: u32 = 0x0312;
const MOD_NOREPEAT: u32 = 0x4000;

/// 自定义窗口过程（拦截 WM_HOTKEY）
extern "system" fn hotkey_wndproc(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> isize {
    if msg == WM_HOTKEY {
        let hotkey_id = wparam as u32;
        if hotkey_id == HOTKEY_ID {
            HOTKEY_TRIGGERED.store(true, Ordering::SeqCst);
            return 0;
        }
    }

    // 调用原始窗口过程
    let original_ptr = ORIGINAL_WNDPROC.load(Ordering::SeqCst) as isize;
    unsafe { CallWindowProcW(original_ptr, hwnd, msg, wparam, lparam) }
}

/// 设置热键（子类化窗口 + 注册热键）
fn setup_hotkey(hwnd: isize) -> (String, u32) {
    // 保存原始窗口过程
    let original = unsafe { GetWindowLongPtrW(hwnd, GWLP_WNDPROC) };
    ORIGINAL_WNDPROC.store(original as *mut c_void, Ordering::SeqCst);

    // 子类化窗口
    unsafe {
        SetWindowLongPtrW(hwnd, GWLP_WNDPROC, hotkey_wndproc as *const () as isize);
    }

    // 注册默认热键 F8
    let vk_f8 = 0x77;
    let result = unsafe { RegisterHotKey(hwnd, HOTKEY_ID as i32, MOD_NOREPEAT, vk_f8) };
    if result != 0 {
        return ("F8".to_string(), vk_f8);
    }

    // F8 被占用
    ("F8(失败)".to_string(), 0)
}

struct MyApp {
    windows: Vec<WindowInfo>,
    filtered: Vec<WindowInfo>,
    search: String,
    tree_mode: bool,
    locate_mode: bool,
    expanded: Vec<bool>,
    message: String,
    locate_dragging: bool,  // 是否正在拖拽定位
    // 热键相关字段
    hotkey_key: String,
    hotkey_vk: u32,
    hotkey_editing: bool,
    hotkey_edit_text: String,
    hwnd: Option<isize>,
    hotkey_setup_done: bool,  // 是否已设置热键
}

impl MyApp {
    fn new(_cc: &eframe::CreationContext) -> Self {
        let windows = WindowsApi::enum_windows();
        let len = windows.len();
        Self {
            windows: windows.clone(),
            filtered: windows,
            search: String::new(),
            tree_mode: false,
            locate_mode: false,
            expanded: vec![false; len],
            message: String::new(),
            locate_dragging: false,
            hotkey_key: "F8".to_string(),
            hotkey_vk: 0x77,
            hotkey_editing: false,
            hotkey_edit_text: String::new(),
            hwnd: None,
            hotkey_setup_done: false,
        }
    }

    fn refresh(&mut self) {
        self.windows = WindowsApi::enum_windows();
        self.expanded = vec![false; self.windows.len()];
        self.message.clear();
        self.apply_filter();
    }

    fn apply_filter(&mut self) {
        if self.search.is_empty() {
            self.filtered = self.windows.clone();
        } else {
            let lower = self.search.to_lowercase();
            self.filtered = self.windows
                .iter()
                .filter(|w| w.search_text().contains(&lower))
                .cloned()
                .collect();
        }
    }

    fn copy_class(class: &str) {
        if let Ok(mut clip) = Clipboard::new() {
            let _ = clip.set_text(class);
        }
    }

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
            unsafe { UnregisterHotKey(hwnd, HOTKEY_ID as i32) };

            // 注册新热键
            let success = unsafe { RegisterHotKey(hwnd, HOTKEY_ID as i32, MOD_NOREPEAT, vk) } != 0;

            if success {
                self.hotkey_key = vk_to_string(vk);
                self.hotkey_vk = vk;
                self.message.clear();
            } else {
                self.message = format!("快捷键 {} 注册失败（已被占用）", new_key);
                // 尝试恢复之前的热键
                if self.hotkey_vk > 0 {
                    unsafe { RegisterHotKey(hwnd, HOTKEY_ID as i32, MOD_NOREPEAT, self.hotkey_vk) };
                }
            }
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // 第一帧设置热键
        if !self.hotkey_setup_done {
            use raw_window_handle::{HasWindowHandle, RawWindowHandle};
            if let Ok(handle) = frame.window_handle() {
                let raw = handle.as_raw();
                if let RawWindowHandle::Win32(win32) = raw {
                    let hwnd = win32.hwnd.get() as isize;
                    self.hwnd = Some(hwnd);
                    let (hotkey_key, hotkey_vk) = setup_hotkey(hwnd);
                    self.hotkey_key = hotkey_key;
                    self.hotkey_vk = hotkey_vk;
                    if self.hotkey_key.contains("(失败)") {
                        self.message = "F8 已被占用，请点击⚙修改快捷键".to_string();
                    }
                }
            }
            self.hotkey_setup_done = true;
        }

        // 检测热键触发
        if HOTKEY_TRIGGERED.load(Ordering::SeqCst) {
            HOTKEY_TRIGGERED.store(false, Ordering::SeqCst);
            if !self.locate_mode {
                self.locate_mode = true;
                self.locate_dragging = false;
                self.message = "按住鼠标左键或右键拖动到目标窗口，松开后定位".to_string();
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // 顶部工具栏
            ui.horizontal(|ui| {
                // 搜索框
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.search)
                        .desired_width(280.0)
                        .hint_text("搜索窗口...")
                );

                if response.changed() {
                    self.apply_filter();
                }

                // 定位按钮 - 显示一个带 + 图标的按钮
                let locate_text = if self.locate_mode { "⊕ 定位中..." } else { "⊕ 定位" };
                let locate_btn = ui.button(locate_text);
                if locate_btn.clicked() {
                    self.locate_mode = !self.locate_mode;
                    self.locate_dragging = false;
                    if self.locate_mode {
                        self.message = "按住鼠标左键或右键拖动到目标窗口，松开后定位".to_string();
                    } else {
                        self.message.clear();
                    }
                }

                // 树形切换
                let btn_text = if self.tree_mode { "列表" } else { "树形" };
                if ui.button(btn_text).clicked() {
                    self.tree_mode = !self.tree_mode;
                    self.expanded = vec![false; self.filtered.len()];
                }

                // 刷新
                if ui.button("刷新").clicked() {
                    self.refresh();
                }

                // 重置
                if ui.button("重置").clicked() {
                    self.search.clear();
                    self.message.clear();
                    self.refresh();  // 重新获取所有窗口
                }

                // 定位快捷键显示和编辑
                ui.horizontal(|ui| {
                    ui.label("定位快捷键:");
                    if self.hotkey_editing {
                        let resp = ui.add(
                            egui::TextEdit::singleline(&mut self.hotkey_edit_text)
                                .desired_width(40.0)
                                .hint_text("F1-F12/A-Z")
                        );
                        // Enter 确认
                        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            let key = self.hotkey_edit_text.clone();
                            self.change_hotkey(&key);
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
            });

            ui.separator();

            // 显示状态消息
            if !self.message.is_empty() {
                ui.label(&self.message);
            }

            // 数据显示
            egui::ScrollArea::vertical().show(ui, |ui| {
                if self.tree_mode {
                    // 树形模式
                    for (i, win) in self.filtered.iter().enumerate() {
                        let title = if win.title.is_empty() { "<无标题>" } else { &win.title };
                        let header = format!("{} [{}:{} / {}]", title, win.process_name, win.pid, win.class_name);

                        ui.horizontal(|ui| {
                            if ui.small_button(if self.expanded.get(i).copied().unwrap_or(false) { "▼" } else { "▶" }).clicked() {
                                if let Some(exp) = self.expanded.get_mut(i) {
                                    *exp = !*exp;
                                    if *exp {
                                        // 延迟加载子窗口
                                        WindowsApi::load_children(win);
                                    }
                                }
                            }
                            ui.label(&header);
                        });

                        // 显示子窗口
                        if self.expanded.get(i).copied().unwrap_or(false) {
                            for child in win.children.borrow().iter() {
                                let child_title = if child.title.is_empty() { "<无标题>" } else { &child.title };
                                ui.horizontal(|ui| {
                                    ui.label("    ");
                                    ui.label(format!("{} [{}]", child_title, child.class_name));
                                });
                            }
                        }
                    }
                } else {
                    // 列表模式 - 使用表格
                    use egui_extras::{Column, TableBuilder};

                    TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(Column::auto().at_least(200.0).at_most(400.0).clip(true))  // 标题 - 固定最大宽度
                        .column(Column::auto().at_least(120.0))   // 进程名称
                        .column(Column::auto().at_least(60.0))    // PID
                        .column(Column::remainder().at_least(200.0).clip(true)) // Class Name
                        .header(20.0, |mut header| {
                            header.col(|ui| { ui.strong("标题"); });
                            header.col(|ui| { ui.strong("进程名称"); });
                            header.col(|ui| { ui.strong("PID"); });
                            header.col(|ui| { ui.strong("Class Name"); });
                        })
                        .body(|mut body| {
                            for win in &self.filtered {
                                let title = if win.title.is_empty() { "<无标题>" } else { &win.title };
                                body.row(18.0, |mut row| {
                                    row.col(|ui| {
                                        // 标题列：显示文本，clip(true) 会自动添加 tooltip
                                        ui.label(title);
                                    });
                                    row.col(|ui| { ui.label(&win.process_name); });
                                    row.col(|ui| { ui.label(win.pid.to_string()); });
                                    row.col(|ui| {
                                        // Class Name 列：点击复制
                                        let resp = ui.selectable_label(false, &win.class_name);
                                        if resp.clicked() {
                                            Self::copy_class(&win.class_name);
                                            self.message = format!("Class Name '{}' 已复制", win.class_name);
                                        }
                                    });
                                });
                            }
                        });
                }
            });
        });

        // 定位模式检测 - SPY++ 风格
        if self.locate_mode {
            ctx.request_repaint();

            // 设置十字光标
            use windows::Win32::UI::WindowsAndMessaging::{SetCursor, LoadCursorW, IDC_CROSS};
            use windows::Win32::Foundation::HINSTANCE;

            unsafe {
                let hcursor = LoadCursorW(HINSTANCE::default(), IDC_CROSS).unwrap();
                SetCursor(hcursor);
            }

            use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, WindowFromPoint};
            use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON, VK_RBUTTON};
            use windows::Win32::Foundation::POINT;

            // 检测鼠标左键状态
            let left_button_down = unsafe { GetAsyncKeyState(VK_LBUTTON.0 as i32) < 0 };
            let right_button_down = unsafe { GetAsyncKeyState(VK_RBUTTON.0 as i32) < 0 };
            let any_button_down = left_button_down || right_button_down;

            // 按下时开始拖拽
            if any_button_down && !self.locate_dragging {
                self.locate_dragging = true;
                self.message = "拖动到目标窗口，松开鼠标...".to_string();
            }

            // 松开时完成定位
            if !any_button_down && self.locate_dragging {
                let mut pt = POINT { x: 0, y: 0 };
                unsafe { GetCursorPos(&mut pt).ok() };
                let hwnd = unsafe { WindowFromPoint(pt) };

                if !hwnd.is_invalid() {
                    let win = WindowsApi::create_window_info(hwnd.0 as isize);

                    // 清空列表，只保留定位到的窗口
                    self.windows = vec![win.clone()];
                    self.filtered = vec![win.clone()];
                    self.expanded = vec![false];

                    self.message = format!("已定位：{}", win.process_name);
                }

                self.locate_mode = false;
                self.locate_dragging = false;
            }
        }
    }
}

/// 解析 --class 命令行参数
fn parse_class_arg() -> Option<String> {
    let args: Vec<String> = std::env::args().collect();

    for i in 1..args.len() {
        // --class="xxxx" 形式
        if args[i].starts_with("--class=") {
            let class_name = args[i].split('=').nth(1).unwrap_or("");
            if !class_name.is_empty() {
                return Some(class_name.to_string());
            }
        }
        // --class xxxx 形式
        if args[i] == "--class" && i + 1 < args.len() {
            let class_name = &args[i + 1];
            if !class_name.is_empty() {
                return Some(class_name.clone());
            }
        }
    }

    None
}

/// 查询窗口并输出 JSON（递归搜索所有窗口，包括子窗口）
fn query_and_output(class_name: &str) {
    let windows = WindowsApi::enum_windows();

    // 递归搜索
    let found = find_window_by_class(&windows, class_name);

    if let Some(win) = found {
        println!("{}", format_json(&win));
    } else {
        println!("null");
    }
}

/// 递归查找指定类名的窗口（包括子窗口）
fn find_window_by_class(windows: &[WindowInfo], class_name: &str) -> Option<WindowInfo> {
    // 先检查当前层级
    for win in windows {
        if win.class_name == class_name {
            return Some(win.clone());
        }

        // 加载子窗口并递归检查
        WindowsApi::load_children(win);
        if let Some(found) = find_window_by_class(win.children.borrow().as_slice(), class_name) {
            return Some(found);
        }
    }
    None
}

/// 手动构建 JSON 输出
fn format_json(win: &WindowInfo) -> String {
    format!(
        "{{\"hwnd\": {}, \"title\": \"{}\", \"class_name\": \"{}\", \"pid\": {}, \"process_name\": \"{}\"}}",
        win.hwnd,
        escape_json(&win.title),
        escape_json(&win.class_name),
        win.pid,
        escape_json(&win.process_name)
    )
}

/// JSON 字符串转义
fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}