use std::ffi::c_void;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::rc::Rc;

use crate::window_info::WindowInfo;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::{EnumChildWindows, EnumWindows};

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

        unsafe { GetWindowThreadProcessId(HWND(hwnd as *mut c_void), None) }
    }

    /// 根据PID获取进程名称
    pub fn get_process_name(pid: u32) -> String {
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION};
        use windows::Win32::System::ProcessStatus::GetProcessImageFileNameW;

        let process_handle = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION, false, pid) };
        let Ok(handle) = process_handle else {
            return "<无法访问>".to_string();
        };

        if handle.is_invalid() {
            return "<无法访问>".to_string();
        }

        let mut buffer: [u16; 512] = [0; 512];
        let len = unsafe { GetProcessImageFileNameW(handle, &mut buffer) };

        unsafe { CloseHandle(handle).ok() };

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

    /// 遍历所有顶层窗口
    pub fn enum_windows() -> Vec<Rc<WindowInfo>> {
        let mut windows: Vec<Rc<WindowInfo>> = Vec::new();

        unsafe {
            let _ = EnumWindows(
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
            let _ = EnumChildWindows(
                HWND(parent_hwnd as *mut c_void),
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
    let info = WindowsApi::create_window_info(hwnd.0 as isize);
    windows.push(info);
    BOOL(1) // 继续枚举
}

/// EnumChildWindows 回调函数
extern "system" fn enum_child_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let children = unsafe { &mut *(lparam.0 as *mut Vec<Rc<WindowInfo>>) };
    let info = WindowsApi::create_window_info(hwnd.0 as isize);
    children.push(info);
    BOOL(1) // 继续枚举
}