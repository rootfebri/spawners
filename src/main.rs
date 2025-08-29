use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use std::thread;
use std::time::Duration;
use windows::Win32::Foundation::{HWND, LPARAM, POINT};
use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow};
use windows::Win32::System::Threading::{GetCurrentProcessId, Sleep};
use windows::Win32::UI::Shell::{SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, ShellExecuteExW};
use windows::Win32::UI::WindowsAndMessaging::{
  EnumWindows, GetCursorPos, GetForegroundWindow, GetWindowThreadProcessId, MoveWindow, SW_HIDE, SW_RESTORE, SW_SHOW,
  ShowWindow,
};
use windows::core::{BOOL, PCWSTR};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  println!("App Spawner - Safe Mode");
  println!("WARNING: This will spawn multiple application instances!");
  println!("Press Ctrl+C now if you're not ready...\n");
  thread::sleep(Duration::from_secs(3));

  // Get user input
  let program = get_input("Enter program path to run (e.g., notepad.exe):")?;
  let count: usize = get_input("Number of programs to spawn:")?.parse()?;
  let max_h_stack: usize = get_input("Maximum horizontal stack:")?.parse()?;
  let max_v_stack: usize = get_input("Maximum vertical stack:")?.parse()?;
  let spacing: i32 = get_input("Spacing between windows (pixels, negative for overlap):")?.parse()?;

  // Safety validation
  if count > 20 {
    println!("WARNING: Spawning more than 20 windows can cause system instability!");
    let confirm = get_input("Type 'CONFIRM' to continue:")?;
    if confirm != "CONFIRM" {
      println!("Operation cancelled");
      return Ok(());
    }
  }

  // Get monitor information
  let monitor_info = get_monitor_info()?;
  let work_area = monitor_info.rcWork;

  // Get starting position from user
  println!("\nMove your mouse to the desired starting position and press Enter...");
  let _ = get_input("Press Enter when ready")?;
  let start_pos = get_cursor_position()?;

  // Validate position is on screen
  if start_pos.x < work_area.left
    || start_pos.x > work_area.right
    || start_pos.y < work_area.top
    || start_pos.y > work_area.bottom
  {
    return Err("Starting position is outside the screen work area".into());
  }

  // Calculate grid positions
  let window_width = 800; // Default window size
  let window_height = 600;

  let mut positions = Vec::new();
  let mut current_x = start_pos.x;
  let mut current_y = start_pos.y;
  let mut h_count = 0;
  let mut v_count = 0;

  for i in 0..count {
    positions.push((current_x, current_y));

    h_count += 1;
    if h_count >= max_h_stack {
      // Move to next row
      h_count = 0;
      v_count += 1;
      current_x = start_pos.x;
      current_y += window_height - spacing;

      if v_count >= max_v_stack {
        println!("Reached maximum vertical stack limit at {} windows", i + 1);
        break;
      }
    } else {
      // Move horizontally
      current_x += window_width - spacing;
    }
  }

  println!("\nSpawning {} windows...", positions.len());

  // Spawn applications
  let mut handles = Vec::new();
  for (i, &(x, y)) in positions.iter().enumerate() {
    println!("Spawning window {} at ({}, {})", i + 1, x, y);
    match spawn_program(&program, i == 0) {
      Ok(handle) => {
        handles.push((handle, x, y));
        thread::sleep(Duration::from_millis(500)); // Allow window to initialize
      }
      Err(e) => {
        eprintln!("Failed to spawn window {}: {}", i + 1, e);
        break;
      }
    }
  }

  // Position windows
  println!("\nPositioning {} windows...", handles.len());
  for (i, (hwnd, x, y)) in handles.iter().enumerate() {
    match position_window(*hwnd, *x, *y, window_width, window_height) {
      Ok(_) => println!("Positioned window {} at ({}, {})", i + 1, x, y),
      Err(e) => eprintln!("Failed to position window {}: {}", i + 1, e),
    }
    unsafe {
      Sleep(100);
    } // Small delay between positioning
  }

  println!("\nOperation completed! Spawned {} windows.", handles.len());
  println!("Close the windows manually or press Enter to exit...");
  let _ = get_input("");
  Ok(())
}

fn get_input(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
  use std::io::{self, Write};

  print!("{} ", prompt);
  io::stdout().flush()?;

  let mut input = String::new();
  io::stdin().read_line(&mut input)?;
  Ok(input.trim().to_string())
}

fn get_cursor_position() -> Result<POINT, Box<dyn std::error::Error>> {
  let mut point = POINT { x: 0, y: 0 };
  unsafe {
    GetCursorPos(&mut point)?;
  }
  Ok(point)
}

fn get_monitor_info() -> Result<MONITORINFO, Box<dyn std::error::Error>> {
  let hwnd = unsafe { GetForegroundWindow() };
  let hmonitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };

  let mut monitor_info = MONITORINFO {
    cbSize: size_of::<MONITORINFO>() as u32,
    ..Default::default()
  };

  unsafe {
    GetMonitorInfoW(hmonitor, &mut monitor_info).ok()?;
  }

  Ok(monitor_info)
}

fn spawn_program(program: &str, first: bool) -> Result<HWND, Box<dyn std::error::Error>> {
  let program_wide: Vec<u16> = OsStr::new(program).encode_wide().chain(once(0)).collect();

  let mut sei = SHELLEXECUTEINFOW {
    cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
    fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC,
    lpFile: PCWSTR(program_wide.as_ptr()),
    nShow: if first { SW_SHOW.0 } else { SW_HIDE.0 },
    ..Default::default()
  };

  unsafe {
    ShellExecuteExW(&mut sei)?;

    // Wait for window to appear
    for attempt in 0..20 {
      println!("sei HWND is_invalid: {}", sei.hwnd.is_invalid());
      println!("sei HWND is_null: {}", sei.hwnd.0.is_null());
      match find_window_by_process(sei.hProcess) {
        Ok(hwnd) => return Ok(hwnd),
        Err(e) => {
          println!("Waiting for window... (attempt {}/{}) Details: {e}", attempt + 1, 20);
          Sleep(500);
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

fn position_window(hwnd: HWND, x: i32, y: i32, width: i32, height: i32) -> Result<(), Box<dyn std::error::Error>> {
  unsafe {
    ShowWindow(hwnd, SW_RESTORE).ok()?;
    MoveWindow(hwnd, x, y, width, height, true)?;
  }
  Ok(())
}
