use std::ffi::c_void;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

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

    /// 判断窗口是否可见
    pub fn is_window_visible(hwnd: isize) -> bool {
        use windows::Win32::UI::WindowsAndMessaging::IsWindowVisible;
        use windows::Win32::Foundation::HWND;

        unsafe { IsWindowVisible(HWND(hwnd as *mut c_void)).as_bool() }
    }
}