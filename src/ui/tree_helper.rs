use crate::window_info::WindowInfo;
use crate::windows_api::WindowsApi;
use std::rc::Rc;

/// 树形视图辅助工具
///
/// 提供树形视图的延迟加载和节点管理功能
pub struct TreeHelper;

impl TreeHelper {
    /// 加载子窗口（延迟加载）
    ///
    /// 该方法实现了延迟加载策略，只有在真正需要时才枚举子窗口。
    /// 这避免了启动时遍历所有窗口层级造成的性能开销。
    ///
    /// # 参数
    ///
    /// * `parent` - 父窗口信息的引用计数指针
    pub fn load_children(parent: &Rc<WindowInfo>) {
        // 如果子窗口已经加载，直接返回
        if *parent.children_loaded.borrow() {
            return;
        }

        // 枚举子窗口
        let children = WindowsApi::enum_child_windows(parent.hwnd);

        // 更新父窗口的子窗口列表
        *parent.children.borrow_mut() = children;
        *parent.children_loaded.borrow_mut() = true;
    }

    /// 将窗口信息转换为显示文本（详细信息格式）
    ///
    /// 用于顶层窗口显示，包含进程名、PID 和类名
    ///
    /// # 参数
    ///
    /// * `win` - 窗口信息引用
    ///
    /// # 返回值
    ///
    /// 格式化的窗口显示文本
    pub fn format_window_detail(win: &WindowInfo) -> String {
        if win.title.is_empty() {
            format!(
                "<无标题> [{}:{} / {}]",
                win.process_name, win.pid, win.class_name
            )
        } else {
            format!(
                "{} [{}:{} / {}]",
                win.title, win.process_name, win.pid, win.class_name
            )
        }
    }

    /// 将窗口信息转换为显示文本（简洁格式）
    ///
    /// 用于子窗口显示，只包含标题和类名
    ///
    /// # 参数
    ///
    /// * `win` - 窗口信息引用
    ///
    /// # 返回值
    ///
    /// 格式化的窗口显示文本
    pub fn format_window_simple(win: &WindowInfo) -> String {
        if win.title.is_empty() {
            format!("<无标题> [{}]", win.class_name)
        } else {
            format!("{} [{}]", win.title, win.class_name)
        }
    }

    /// 递归计算窗口树的总节点数
    ///
    /// 用于显示统计信息或预估加载时间
    ///
    /// # 参数
    ///
    /// * `window` - 窗口信息引用
    /// * `include_children` - 是否包含子窗口计数
    ///
    /// # 返回值
    ///
    /// 窗口树的总节点数
    pub fn count_window_nodes(window: &Rc<WindowInfo>, include_children: bool) -> usize {
        if !include_children {
            return 1;
        }

        // 确保子窗口已加载
        Self::load_children(window);

        let mut count = 1;
        for child in window.children.borrow().iter() {
            count += Self::count_window_nodes(child, true);
        }
        count
    }

    /// 查找窗口树中指定句柄的窗口
    ///
    /// # 参数
    ///
    /// * `windows` - 窗口列表
    /// * `hwnd` - 目标窗口句柄
    ///
    /// # 返回值
    ///
    /// 找到的窗口信息，如果未找到则返回 None
    pub fn find_window_by_hwnd(windows: &[Rc<WindowInfo>], hwnd: isize) -> Option<Rc<WindowInfo>> {
        for win in windows {
            if win.hwnd == hwnd {
                return Some(Rc::clone(win));
            }

            // 确保子窗口已加载
            Self::load_children(win);

            // 递归搜索子窗口
            if let Some(found) = Self::find_window_by_hwnd(&win.children.borrow(), hwnd) {
                return Some(found);
            }
        }
        None
    }

    /// 检查窗口是否有子窗口
    ///
    /// # 参数
    ///
    /// * `window` - 窗口信息引用
    ///
    /// # 返回值
    ///
    /// 如果有子窗口返回 true，否则返回 false
    pub fn has_children(window: &Rc<WindowInfo>) -> bool {
        Self::load_children(window);
        !window.children.borrow().is_empty()
    }

    /// 获取窗口的完整路径标题
    ///
    /// 从根窗口到指定窗口的标题路径，用 " > " 分隔
    ///
    /// # 参数
    ///
    /// * `windows` - 根窗口列表
    /// * `target_hwnd` - 目标窗口句柄
    ///
    /// # 返回值
    ///
    /// 完整路径字符串，如果未找到则返回空字符串
    pub fn get_window_path(windows: &[Rc<WindowInfo>], target_hwnd: isize) -> String {
        fn build_path(
            window: &Rc<WindowInfo>,
            target: isize,
            path: &mut String,
        ) -> bool {
            if window.hwnd == target {
                if !path.is_empty() {
                    path.push_str(" > ");
                }
                path.push_str(&if window.title.is_empty() {
                    format!("<无标题>")
                } else {
                    window.title.clone()
                });
                return true;
            }

            TreeHelper::load_children(window);
            for child in window.children.borrow().iter() {
                let old_len = path.len();
                if !path.is_empty() {
                    path.push_str(" > ");
                }
                path.push_str(&if window.title.is_empty() {
                    format!("<无标题>")
                } else {
                    window.title.clone()
                });

                if build_path(child, target, path) {
                    return true;
                }

                // 回溯
                path.truncate(old_len);
            }
            false
        }

        let mut path = String::new();
        for win in windows {
            if build_path(win, target_hwnd, &mut path) {
                return path;
            }
        }
        String::new()
    }
}

impl Default for TreeHelper {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_window_detail() {
        let win = WindowInfo::new(
            12345,
            "测试窗口".to_string(),
            "TestClass".to_string(),
            1000,
            "test.exe".to_string(),
        );

        let text = TreeHelper::format_window_detail(&win);
        assert!(text.contains("测试窗口"));
        assert!(text.contains("test.exe"));
        assert!(text.contains("1000"));
        assert!(text.contains("TestClass"));
    }

    #[test]
    fn test_format_window_simple() {
        let win = WindowInfo::new(
            12345,
            "测试窗口".to_string(),
            "TestClass".to_string(),
            1000,
            "test.exe".to_string(),
        );

        let text = TreeHelper::format_window_simple(&win);
        assert!(text.contains("测试窗口"));
        assert!(text.contains("TestClass"));
        assert!(!text.contains("test.exe"));
        assert!(!text.contains("1000"));
    }

    #[test]
    fn test_format_window_empty_title() {
        let win = WindowInfo::new(
            12345,
            String::new(),
            "TestClass".to_string(),
            1000,
            "test.exe".to_string(),
        );

        let detail = TreeHelper::format_window_detail(&win);
        assert!(detail.contains("<无标题>"));

        let simple = TreeHelper::format_window_simple(&win);
        assert!(simple.contains("<无标题>"));
    }
}