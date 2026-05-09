use std::cell::{Cell, RefCell};

/// 窗口信息结构体
#[derive(Debug, Clone)]
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
    pub children: RefCell<Vec<WindowInfo>>,
    /// 子窗口是否已加载
    pub children_loaded: Cell<bool>,
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
            children_loaded: Cell::new(false),
        }
    }

    /// 创建用于搜索的简化字符串
    pub fn search_text(&self) -> String {
        format!("{} {} {} {}", self.title, self.process_name, self.pid, self.class_name).to_lowercase()
    }
}