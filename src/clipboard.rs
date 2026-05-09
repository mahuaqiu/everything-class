// src/clipboard.rs
use clipboard_win::{formats, set_clipboard};

/// 剪贴板操作封装
pub struct ClipboardHelper;

impl ClipboardHelper {
    /// 复制文本到剪贴板
    pub fn copy_text(text: &str) -> bool {
        set_clipboard(formats::Unicode, text).is_ok()
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