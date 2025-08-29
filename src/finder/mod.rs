use std::{cell::RefCell, collections::HashSet, ffi::OsString, os::windows::ffi::OsStringExt};
use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
use windows::Win32::{
  Foundation::{BOOL, HWND, LPARAM},
  System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
  },
  UI::WindowsAndMessaging::{
    EnumWindows, GA_ROOT, GWL_EXSTYLE, GetAncestor, GetWindowLongPtrW, IsWindowVisible, WS_EX_TOOLWINDOW,
  },
};

fn widestr_to_string(buf: &[u16]) -> String {
  let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
  OsString::from_wide(&buf[..len]).to_string_lossy().into_owned()
}

// Kumpulkan semua PID yang exe-nya == target_exe (case-insensitive, hanya nama file, bukan full path)
fn pids_by_exe_name(target_exe: &str) -> windows::core::Result<HashSet<u32>> {
  unsafe {
    let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;
    let mut entry = PROCESSENTRY32W::default();
    entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;

    let mut pids = HashSet::new();

    if Process32FirstW(snap, &mut entry).is_ok() {
      loop {
        let name = widestr_to_string(&entry.szExeFile).to_lowercase();
        if name.contains(target_exe) {
          pids.insert(entry.th32ProcessID);
        }
        if !Process32NextW(snap, &mut entry).is_ok() {
          break;
        }
      }
    }
    Ok(pids)
  }
}

fn is_top_level_main(hwnd: HWND) -> bool {
  unsafe {
    if !IsWindowVisible(hwnd).as_bool() {
      return false;
    }
    if GetAncestor(hwnd, GA_ROOT) != hwnd {
      return false;
    }
    let exstyle = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
    if (exstyle & WS_EX_TOOLWINDOW.0 as isize) != 0 {
      return false;
    }
  }
  true
}

/// Ambil semua HWND top-level yang dimiliki proses bernama `exe_name` (mis. "chrome.exe" / "foo.exe")
pub fn hwnds_for_exe(exe_name: &str) -> windows::core::Result<Vec<HWND>> {
  let pids = pids_by_exe_name(exe_name)?;
  if pids.is_empty() {
    return Ok(Vec::new());
  }

  thread_local! {
      static COLLECT: RefCell<Vec<HWND>> = RefCell::new(Vec::new());
      static PIDS: RefCell<HashSet<u32>> = RefCell::new(HashSet::new());
  }

  PIDS.with(|cell| {
    *cell.borrow_mut() = pids;
  });
  COLLECT.with(|cell| cell.borrow_mut().clear());

  unsafe extern "system" fn enum_proc(hwnd: HWND, _lparam: LPARAM) -> BOOL {
    if !is_top_level_main(hwnd) {
      return BOOL(1);
    }
    let mut pid: u32 = 0;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    let belongs = PIDS.with(|cell| cell.borrow().contains(&pid));
    if belongs {
      COLLECT.with(|cell| cell.borrow_mut().push(hwnd));
    }
    BOOL(1)
  }

  unsafe {
    _ = EnumWindows(Some(enum_proc), LPARAM(0));
  }

  let result = COLLECT.with(|cell| cell.borrow().clone());
  Ok(result)
}
