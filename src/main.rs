// Windows 上隐藏控制台窗口
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod window_info;
mod windows_api;

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

struct MyApp {
    windows: Vec<WindowInfo>,
    filtered: Vec<WindowInfo>,
    search: String,
    tree_mode: bool,
    locate_mode: bool,
    expanded: Vec<bool>,
    message: String,
    locate_dragging: bool,  // 是否正在拖拽定位
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
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                        self.message = "按住鼠标左键拖动到目标窗口，松开后定位".to_string();
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
                    self.apply_filter();
                }
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
            use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};
            use windows::Win32::Foundation::POINT;

            // 检测鼠标左键状态
            let left_button_down = unsafe { GetAsyncKeyState(VK_LBUTTON.0 as i32) < 0 };

            // 按下时开始拖拽
            if left_button_down && !self.locate_dragging {
                self.locate_dragging = true;
                self.message = "拖动到目标窗口，松开鼠标...".to_string();
            }

            // 松开时完成定位
            if !left_button_down && self.locate_dragging {
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