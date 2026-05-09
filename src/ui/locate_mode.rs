use std::ffi::c_void;
use std::rc::Rc;

use crate::window_info::WindowInfo;
use crate::windows_api::WindowsApi;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::KeyboardAndMouse::{ReleaseCapture, SetCapture};
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, LoadCursorW, SetCursor, WindowFromPoint, IDC_CROSS,
};

/// 定位模式处理
#[derive(Clone)]
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
            // 设置十字准星光标
            if let Ok(cursor) = LoadCursorW(None, IDC_CROSS) {
                SetCursor(cursor);
            }
            // 捕获鼠标
            let _ = SetCapture(HWND(window_hwnd as *mut c_void));
        }
    }

    /// 结束定位模式
    pub fn stop(&mut self) {
        self.active = false;
        unsafe {
            let _ = ReleaseCapture();
            // 恢复默认光标
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
            GetCursorPos(&mut point).ok()?;
        }

        let hwnd = unsafe { WindowFromPoint(point) };
        if hwnd.is_invalid() {
            return None;
        }

        Some(WindowsApi::create_window_info(hwnd.0 as isize))
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