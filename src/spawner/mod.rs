use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::prelude::*;
use std::ptr;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
use windows::Win32::System::Threading::{GetCurrentProcessId, Sleep};
use windows::Win32::UI::Shell::{SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, ShellExecuteExW};
use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowThreadProcessId, SW_HIDE, SW_SHOW};
use windows::core::PCWSTR;

pub fn spawn_program(program: &str, _first: bool) -> Result<HWND, Box<dyn std::error::Error>> {
  let program_wide: Vec<u16> = OsStr::new(program).encode_wide().chain(once(0)).collect();

  let mut sei = SHELLEXECUTEINFOW {
    cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
    fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC,
    lpFile: PCWSTR(program_wide.as_ptr()),
    nShow: SW_SHOW.0,
    ..Default::default()
  };

  unsafe {
    ShellExecuteExW(&mut sei)?;

    // Wait for window to appear
    for attempt in 0..20 {
      dbg!(sei.hProcess);
      dbg!(sei.hProcess.0.is_null());
      match find_window_by_process(sei.hProcess) {
        Ok(hwnd) => {
          return Ok(hwnd);
        }
        Err(e) => {
          println!("Waiting for window... (attempt {}/{}) Details: {e}", attempt + 1, 20);
          Sleep(1500);
        }
      }
    }
  }

  Err("Failed to get window handle after 20 attempts".into())
}

fn find_window_by_process(process: windows::Win32::Foundation::HANDLE) -> Result<HWND, Box<dyn std::error::Error>> {
  let target_process_id = unsafe { windows::Win32::System::Threading::GetProcessId(process) };
  let current_process_id = unsafe { GetCurrentProcessId() };

  // Don't try to manipulate windows of this process
  if target_process_id == current_process_id {
    return Err("Cannot manipulate windows of the current process".into());
  }

  let mut result = HWND(ptr::null_mut());

  unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let (target_process_id, result_ptr) = unsafe { *(lparam.0 as *const (u32, *mut HWND)) };
    let mut window_pid = 0;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut window_pid)) };

    if window_pid == target_process_id {
      unsafe { *result_ptr = hwnd }
      BOOL(0) // Stop enumeration
    } else {
      BOOL(1) // Continue
    }
  }

  let params = (target_process_id, &mut result as *mut HWND);
  unsafe {
    _ = EnumWindows(Some(enum_windows_proc), LPARAM(&params as *const _ as isize));
  }

  if result.0.is_null() {
    Err("Window not found".into())
  } else {
    Ok(result)
  }
}
