use std::{
  cell::RefCell,
  collections::HashSet,
  ffi::{OsStr, OsString},
  os::windows::ffi::OsStringExt,
  path::Path,
};
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
pub fn pids_by_exe_name(target_exe: &str) -> windows::core::Result<HashSet<u32>> {
  // Normalize input: accept full paths or bare names; compare case-insensitively; tolerate missing .exe
  let target_basename: &OsStr = Path::new(target_exe)
    .file_name()
    .unwrap_or_else(|| OsStr::new(target_exe));
  let target = target_basename.to_string_lossy().to_string();
  let target = target.to_lowercase();
  let target_with_exe = if target.ends_with(".exe") {
    target.clone()
  } else {
    format!("{target}.exe")
  };

  unsafe {
    let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;
    let mut entry = PROCESSENTRY32W::default();
    entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;

    let mut pids = HashSet::new();

    if Process32FirstW(snap, &mut entry).is_ok() {
      loop {
        let name = widestr_to_string(&entry.szExeFile).to_lowercase();
        // Prefer exact match on exe name (case-insensitive). As a fallback, allow substring match on the normalized target.
        if name == target || name == target_with_exe || (!target.is_empty() && name.contains(&target)) {
          pids.insert(entry.th32ProcessID);
        }
        if Process32NextW(snap, &mut entry).is_err() {
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
      static COLLECT: RefCell<Vec<HWND>> = const { RefCell::new(Vec::new()) };
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

/// Enumerate top-level HWNDs for a specific set of PIDs
pub fn hwnds_for_pids(pids: &HashSet<u32>) -> windows::core::Result<Vec<HWND>> {
  if pids.is_empty() {
    return Ok(Vec::new());
  }

  thread_local! {
      static COLLECT: RefCell<Vec<HWND>> = const { RefCell::new(Vec::new()) };
      static PIDS: RefCell<HashSet<u32>> = RefCell::new(HashSet::new());
  }

  PIDS.with(|cell| {
    *cell.borrow_mut() = pids.clone();
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

/// Collect all descendant PIDs (children, grandchildren, ...) of the given root PIDs using a single snapshot
pub fn descendant_pids(root_pids: &HashSet<u32>) -> windows::core::Result<HashSet<u32>> {
  if root_pids.is_empty() {
    return Ok(HashSet::new());
  }
  unsafe {
    let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;
    let mut entry = PROCESSENTRY32W::default();
    entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;

    // Collect (pid, ppid) pairs
    let mut proc_pairs: Vec<(u32, u32)> = Vec::new();
    if Process32FirstW(snap, &mut entry).is_ok() {
      loop {
        let pid = entry.th32ProcessID;
        let ppid = entry.th32ParentProcessID;
        proc_pairs.push((pid, ppid));
        if Process32NextW(snap, &mut entry).is_err() {
          break;
        }
      }
    }

    // BFS from roots
    let mut result: HashSet<u32> = HashSet::new();
    let mut queue: Vec<u32> = root_pids.iter().copied().collect();
    while let Some(cur) = queue.pop() {
      for (pid, ppid) in proc_pairs.iter() {
        if *ppid == cur && !root_pids.contains(pid) && result.insert(*pid) {
          queue.push(*pid);
        }
      }
    }
    Ok(result)
  }
}

pub fn get_process_name_by_pid(pid: u32) -> windows::core::Result<String> {
  unsafe {
    let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;
    let mut entry = PROCESSENTRY32W::default();
    entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;

    if Process32FirstW(snap, &mut entry).is_ok() {
      loop {
        if entry.th32ProcessID == pid {
          let name = widestr_to_string(&entry.szExeFile);
          return Ok(name);
        }
        if Process32NextW(snap, &mut entry).is_err() {
          break;
        }
      }
    }
    Err(windows::core::Error::new(
      windows::core::HRESULT(0),
      "Process not found",
    ))
  }
}
