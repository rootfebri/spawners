use anyhow::Result;
use spawners::finder::hwnds_for_exe;
use spawners::get_input;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow};
use windows::Win32::System::Threading::Sleep;
use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, GetForegroundWindow, MoveWindow, SW_RESTORE, ShowWindow};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let program = get_input("Enter program path to run (e.g., notepad.exe or C:\\Windows\\System32\\cmd.exe:")?;
  let prog_path = PathBuf::from(&program);

  if prog_path.is_absolute() && !prog_path.exists() {
    return Err("Program does not exist".to_owned().into());
  }

  let count: usize = get_input("Number of programs to spawn:")?.parse()?;
  let max_h_stack: usize = get_input("Maximum horizontal stack:")?.parse()?;
  let max_v_stack: usize = get_input("Maximum vertical stack:")?.parse()?;
  let spacing: i32 = get_input("Spacing between windows (pixels, negative for overlap):")?.parse()?;

  println!("\nSpawning {} windows...", count);
  for _ in 0..count {
    if let Err(e) = Command::new(&program).spawn() {
      eprintln!("{e}");
    } else {
      thread::sleep(Duration::from_millis(1500)); // Small delay to allow window to appear
      println!("Spawned instance of {program}");
    }
  }

  _ = get_input("Press enter if all Programs have been spawned!");
  let handles = hwnds_for_exe(program.as_ref())?;
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
  let window_width = 308; // Default window size
  let window_height = 265;

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
      current_y += window_height; // No spacing for vertical - side by side

      if v_count >= max_v_stack {
        println!("Reached maximum vertical stack limit at {} windows", i + 1);
        break;
      }
    } else {
      current_x += window_width + spacing; // Add spacing for horizontal
    }
  }

  // Position windows
  println!("\nPositioning {} windows...", handles.len());
  for ((i, hwnd), (x, y)) in handles.iter().enumerate().zip(positions) {
    match position_window(*hwnd, x, y, window_width, window_height) {
      Ok(_) => println!("Positioned window {} at ({}, {})", i + 1, x, y),
      Err(e) => eprintln!("Failed to position window {}: {}", i + 1, e),
    }
    unsafe {
      Sleep(500);
    } // Small delay between positioning
  }

  println!("\nOperation completed! Spawned {} windows.", handles.len());
  println!("Close the windows manually or press Enter to exit...");
  let _ = get_input("");
  Ok(())
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

fn position_window(hwnd: HWND, x: i32, y: i32, width: i32, height: i32) -> Result<(), Box<dyn std::error::Error>> {
  unsafe {
    ShowWindow(hwnd, SW_RESTORE).ok()?;
    MoveWindow(hwnd, x, y, width, height, true)?;
  }
  Ok(())
}
