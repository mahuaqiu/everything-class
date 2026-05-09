mod window_info;
mod windows_api;
mod clipboard;
mod ui;

use native_windows_gui as nwg;
use native_windows_derive as nwd;
use nwd::NwgUi;
use nwg::NativeUi;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use crate::clipboard::ClipboardHelper;
use crate::ui::{AppState, LocateMode, TreeHelper};
use crate::window_info::WindowInfo;

#[derive(Default, NwgUi)]
pub struct MainWindow {
    #[nwg_control(size: (800, 600), title: "Window Handle Finder", position: (100, 100))]
    #[nwg_events(OnInit: [MainWindow::init], OnWindowClose: [MainWindow::close])]
    window: nwg::Window,

    #[nwg_layout(parent: window)]
    layout: nwg::GridLayout,

    // 搜索框
    #[nwg_control(parent: window, focus: true, placeholder_text: Some("搜索窗口..."), size: (200, 25), position: (5, 5))]
    #[nwg_layout_item(layout: layout, col: 0, row: 0)]
    #[nwg_events(OnTextInput: [MainWindow::search_input])]
    search_box: nwg::TextInput,

    // 定位按钮
    #[nwg_control(parent: window, text: "定位", size: (80, 25), position: (210, 5))]
    #[nwg_layout_item(layout: layout, col: 1, row: 0)]
    #[nwg_events(OnButtonClick: [MainWindow::start_locate])]
    locate_btn: nwg::Button,

    // 树形切换按钮
    #[nwg_control(parent: window, text: "树形", size: (80, 25), position: (295, 5))]
    #[nwg_layout_item(layout: layout, col: 2, row: 0)]
    #[nwg_events(OnButtonClick: [MainWindow::toggle_tree_mode])]
    tree_btn: nwg::Button,

    // 刷新按钮
    #[nwg_control(parent: window, text: "刷新", size: (80, 25), position: (380, 5))]
    #[nwg_layout_item(layout: layout, col: 3, row: 0)]
    #[nwg_events(OnButtonClick: [MainWindow::refresh_list])]
    refresh_btn: nwg::Button,

    // ListView（列表模式）
    #[nwg_control(parent: window, size: (790, 560), position: (5, 35))]
    #[nwg_layout_item(layout: layout, col: 0, row: 1, col_span: 4)]
    #[nwg_events(OnListViewDoubleClick: [MainWindow::copy_handle])]
    list_view: nwg::ListView,

    // TreeView（树形模式）
    #[nwg_control(parent: window, size: (790, 560), position: (5, 35))]
    #[nwg_layout_item(layout: layout, col: 0, row: 1, col_span: 4)]
    #[nwg_events(OnTreeViewDoubleClick: [MainWindow::copy_handle])]
    tree_view: nwg::TreeView,

    // 右键菜单
    #[nwg_control(parent: window)]
    context_menu: nwg::Menu,

    #[nwg_control(parent: context_menu, text: "复制Handle")]
    #[nwg_events(OnMenuItemSelected: [MainWindow::menu_copy_handle])]
    menu_copy_h: nwg::MenuItem,

    #[nwg_control(parent: context_menu, text: "复制Class Name")]
    #[nwg_events(OnMenuItemSelected: [MainWindow::menu_copy_class])]
    menu_copy_class: nwg::MenuItem,

    #[nwg_control(parent: context_menu, text: "复制完整信息")]
    #[nwg_events(OnMenuItemSelected: [MainWindow::menu_copy_full])]
    menu_copy_full: nwg::MenuItem,

    #[nwg_control(parent: context_menu)]
    menu_sep: nwg::MenuSeparator,

    #[nwg_control(parent: context_menu, text: "刷新")]
    #[nwg_events(OnMenuItemSelected: [MainWindow::refresh_list])]
    menu_refresh: nwg::MenuItem,

    // 定位模式定时器
    #[nwg_control(parent: window, interval: Duration::from_millis(50))]
    #[nwg_events(OnTimerTick: [MainWindow::check_locate])]
    locate_timer: nwg::AnimationTimer,

    // 应用状态
    data: RefCell<AppState>,
    locate_mode: RefCell<LocateMode>,
    list_items: RefCell<Vec<Rc<WindowInfo>>>,
}

impl MainWindow {
    /// 初始化窗口
    fn init(&self) {
        // 配置 ListView 列
        self.list_view.set_headers_enabled(true);

        // 设置列头
        self.list_view.insert_column("标题");
        self.list_view.insert_column("进程名称");
        self.list_view.insert_column("PID");
        self.list_view.insert_column("Class Name");

        // 初始隐藏 TreeView
        self.tree_view.set_visible(false);

        // 停止定位定时器（初始不运行）
        self.locate_timer.stop();

        // 加载初始数据
        self.refresh_list();

        // 显示提示
        nwg::modal_info_message(
            &self.window,
            "提示",
            "程序已启动\n\n使用说明:\n- 在搜索框输入关键词过滤窗口\n- 点击【定位】按钮后点击目标窗口\n- 点击【树形】按钮切换显示模式\n- 双击列表项复制Handle\n- 右键查看更多选项",
        );
    }

    /// 刷新窗口列表
    fn refresh_list(&self) {
        self.data.borrow_mut().refresh();

        if self.data.borrow().tree_mode {
            self.populate_tree();
        } else {
            self.populate_list(&self.data.borrow().filtered_windows.clone());
        }
    }

    /// 填充 ListView
    fn populate_list(&self, windows: &[Rc<WindowInfo>]) {
        self.list_view.clear();
        self.list_items.borrow_mut().clear();

        for win in windows.iter() {
            let title = if win.title.is_empty() { "<无标题>" } else { &win.title };
            // 使用 insert_items_row 一次性插入整行
            self.list_view.insert_items_row(None, &[
                title,
                &win.process_name,
                &win.pid.to_string(),
                &win.class_name,
            ]);

            // 保存映射
            self.list_items.borrow_mut().push(Rc::clone(win));
        }
    }

    /// 填充 TreeView
    fn populate_tree(&self) {
        self.tree_view.clear();

        for win in &self.data.borrow().filtered_windows {
            self.add_tree_node(None, win);
        }
    }

    /// 递归添加树节点
    fn add_tree_node(&self, parent: Option<&nwg::TreeItem>, win: &Rc<WindowInfo>) {
        let text = TreeHelper::format_window_detail(win);
        // 将 Rc 指针转换为 isize 作为 lParam 存储
        let ptr = Rc::into_raw(Rc::clone(win)) as isize;
        let item = self.tree_view.insert_item_with_param(&text, parent, nwg::TreeInsert::Last, ptr);

        // 延迟加载子窗口
        TreeHelper::load_children(win);
        for child in win.children.borrow().iter() {
            self.add_tree_node(Some(&item), child);
        }
    }

    /// 从 TreeItem 获取 WindowInfo
    fn get_tree_item_info(&self, item: &nwg::TreeItem) -> Option<Rc<WindowInfo>> {
        let ptr = self.tree_view.item_param(item)?;
        if ptr == 0 {
            return None;
        }
        // 从 raw 指针重建 Rc（不增加引用计数）
        let win = unsafe { Rc::from_raw(ptr as *const WindowInfo) };
        // 克隆一份以便安全使用
        let cloned = Rc::clone(&win);
        // 释放重建的 Rc（避免释放内存）
        std::mem::forget(win);
        Some(cloned)
    }

    /// 搜索输入处理
    fn search_input(&self) {
        let text = self.search_box.text();
        self.data.borrow_mut().search_text = text;
        self.data.borrow_mut().apply_filter();

        if self.data.borrow().tree_mode {
            self.populate_tree();
        } else {
            self.populate_list(&self.data.borrow().filtered_windows.clone());
        }
    }

    /// 开始定位模式
    fn start_locate(&self) {
        if self.locate_mode.borrow().is_active() {
            return;
        }

        // 获取窗口句柄
        if let Some(hwnd) = self.window.handle.hwnd() {
            self.locate_mode.borrow_mut().start(hwnd as isize);
            self.locate_btn.set_text("点击目标...");

            // 启动定时器
            self.locate_timer.start();
        }
    }

    /// 定位模式定时器检查
    fn check_locate(&self) {
        if !self.locate_mode.borrow().is_active() {
            self.locate_timer.stop();
            return;
        }

        if let Some(win) = self.locate_mode.borrow().get_window_at_cursor() {
            // 找到窗口，结束定位
            self.locate_mode.borrow_mut().stop();
            self.locate_timer.stop();
            self.locate_btn.set_text("定位");

            // 显示窗口信息
            let title = if win.title.is_empty() { "<无标题>" } else { &win.title };
            nwg::modal_info_message(
                &self.window,
                "定位成功",
                &format!(
                    "Handle: 0x{:08X}\n标题: {}\n类名: {}\nPID: {}\n进程: {}",
                    win.hwnd, title, win.class_name, win.pid, win.process_name
                ),
            );
        }
    }

    /// 切换树形模式
    fn toggle_tree_mode(&self) {
        let mut data = self.data.borrow_mut();
        data.tree_mode = !data.tree_mode;
        let is_tree = data.tree_mode;
        drop(data);

        // 切换显示
        self.list_view.set_visible(!is_tree);
        self.tree_view.set_visible(is_tree);
        self.tree_btn.set_text(if is_tree { "列表" } else { "树形" });

        // 重新加载数据
        if is_tree {
            self.populate_tree();
        } else {
            self.populate_list(&self.data.borrow().filtered_windows.clone());
        }
    }

    /// 双击复制Handle
    fn copy_handle(&self) {
        if self.data.borrow().tree_mode {
            // TreeView 模式
            if let Some(item) = self.tree_view.selected_item() {
                if let Some(win) = self.get_tree_item_info(&item) {
                    if ClipboardHelper::copy_handle(win.hwnd) {
                        self.show_status(&format!("Handle 0x{:08X} 已复制", win.hwnd));
                    }
                }
            }
        } else {
            // ListView 模式 - 获取第一个选中项
            if let Some(idx) = self.list_view.selected_item() {
                if let Some(win) = self.list_items.borrow().get(idx) {
                    if ClipboardHelper::copy_handle(win.hwnd) {
                        self.show_status(&format!("Handle 0x{:08X} 已复制", win.hwnd));
                    }
                }
            }
        }
    }

    /// 菜单：复制Handle
    fn menu_copy_handle(&self) {
        self.copy_handle();
    }

    /// 菜单：复制Class Name
    fn menu_copy_class(&self) {
        if self.data.borrow().tree_mode {
            if let Some(item) = self.tree_view.selected_item() {
                if let Some(win) = self.get_tree_item_info(&item) {
                    if ClipboardHelper::copy_text(&win.class_name) {
                        self.show_status(&format!("Class '{}' 已复制", win.class_name));
                    }
                }
            }
        } else {
            if let Some(idx) = self.list_view.selected_item() {
                if let Some(win) = self.list_items.borrow().get(idx) {
                    if ClipboardHelper::copy_text(&win.class_name) {
                        self.show_status(&format!("Class '{}' 已复制", win.class_name));
                    }
                }
            }
        }
    }

    /// 菜单：复制完整信息
    fn menu_copy_full(&self) {
        if self.data.borrow().tree_mode {
            if let Some(item) = self.tree_view.selected_item() {
                if let Some(win) = self.get_tree_item_info(&item) {
                    if ClipboardHelper::copy_full_info(
                        win.hwnd,
                        &win.title,
                        &win.class_name,
                        win.pid,
                        &win.process_name,
                    ) {
                        self.show_status("完整信息已复制");
                    }
                }
            }
        } else {
            if let Some(idx) = self.list_view.selected_item() {
                if let Some(win) = self.list_items.borrow().get(idx) {
                    if ClipboardHelper::copy_full_info(
                        win.hwnd,
                        &win.title,
                        &win.class_name,
                        win.pid,
                        &win.process_name,
                    ) {
                        self.show_status("完整信息已复制");
                    }
                }
            }
        }
    }

    /// 显示状态信息
    fn show_status(&self, msg: &str) {
        self.window.set_text(&format!("{} - {}", msg, "Window Handle Finder"));
    }

    /// 关闭窗口
    fn close(&self) {
        nwg::stop_thread_dispatch();
    }
}

fn main() {
    // 初始化 native-windows-gui
    nwg::init().expect("Failed to init Native Windows GUI");

    // 构建UI
    let _app = MainWindow::build_ui(Default::default()).expect("Failed to build UI");

    // 运行消息循环
    nwg::dispatch_thread_events();
}