use std::ffi::c_void;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

use crate::window_info::WindowInfo;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::{EnumChildWindows, EnumWindows};

// 直接使用 FFI 定义 OpenProcess 和相关函数（windows crate 封装有问题）
#[link(name = "kernel32")]
extern "system" {
    fn OpenProcess(dwDesiredAccess: u32, bInheritHandle: i32, dwProcessId: u32) -> isize;
    fn CloseHandle(hObject: isize) -> i32;
}

#[link(name = "psapi")]
extern "system" {
    fn GetProcessImageFileNameW(hProcess: isize, lpImageFileName: *mut u16, nSize: u32) -> u32;
}

const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;

/// Windows API 调用封装
pub struct WindowsApi;

impl WindowsApi {
    /// 获取窗口标题
    pub fn get_window_title(hwnd: isize) -> String {
        use windows::Win32::UI::WindowsAndMessaging::GetWindowTextW;
        use windows::Win32::Foundation::HWND;

        let mut buffer: [u16; 512] = [0; 512];
        let len = unsafe { GetWindowTextW(HWND(hwnd as *mut c_void), &mut buffer) };
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
        let len = unsafe { GetClassNameW(HWND(hwnd as *mut c_void), &mut buffer) };
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

        let mut pid: u32 = 0;
        unsafe {
            GetWindowThreadProcessId(HWND(hwnd as *mut c_void), Some(&mut pid));
        }
        pid
    }

    /// 根据PID获取进程名称（使用原始 FFI）
    pub fn get_process_name(pid: u32) -> String {
        let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };

        if handle == 0 {
            return "<无法访问>".to_string();
        }

        let mut buffer: [u16; 512] = [0; 512];
        let len = unsafe { GetProcessImageFileNameW(handle, buffer.as_mut_ptr(), 512) };
        unsafe { CloseHandle(handle) };

        if len == 0 {
            return "<无法访问>".to_string();
        }

        let os_string = OsString::from_wide(&buffer[..len as usize]);
        let full_path = os_string.to_string_lossy();
        full_path.rsplit('\\').next().unwrap_or(&full_path).to_string()
    }

    /// 遍历所有顶层窗口
    pub fn enum_windows() -> Vec<WindowInfo> {
        let mut windows: Vec<WindowInfo> = Vec::new();

        unsafe {
            let _ = EnumWindows(
                Some(enum_windows_callback),
                LPARAM(&mut windows as *mut Vec<WindowInfo> as isize),
            );
        }

        windows
    }

    /// 遍历指定窗口的子窗口
    pub fn enum_child_windows(parent_hwnd: isize) -> Vec<WindowInfo> {
        let mut children: Vec<WindowInfo> = Vec::new();

        unsafe {
            let _ = EnumChildWindows(
                HWND(parent_hwnd as *mut c_void),
                Some(enum_child_windows_callback),
                LPARAM(&mut children as *mut Vec<WindowInfo> as isize),
            );
        }

        children
    }

    /// 创建 WindowInfo 对象
    pub fn create_window_info(hwnd: isize) -> WindowInfo {
        WindowInfo::new(
            hwnd,
            Self::get_window_title(hwnd),
            Self::get_class_name(hwnd),
            Self::get_window_pid(hwnd),
            Self::get_process_name(Self::get_window_pid(hwnd)),
        )
    }

    /// 加载子窗口（延迟加载）
    pub fn load_children(win: &WindowInfo) {
        if win.children_loaded.get() {
            return;
        }
        *win.children.borrow_mut() = Self::enum_child_windows(win.hwnd);
        win.children_loaded.set(true);
    }
}

/// EnumWindows 回调函数
extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let windows = unsafe { &mut *(lparam.0 as *mut Vec<WindowInfo>) };
    let info = WindowsApi::create_window_info(hwnd.0 as isize);
    windows.push(info);
    BOOL(1)
}

/// EnumChildWindows 回调函数
extern "system" fn enum_child_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let children = unsafe { &mut *(lparam.0 as *mut Vec<WindowInfo>) };
    let info = WindowsApi::create_window_info(hwnd.0 as isize);
    children.push(info);
    BOOL(1)
}