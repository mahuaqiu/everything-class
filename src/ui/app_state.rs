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