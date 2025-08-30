use std::error::Error;
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow};
use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, GetForegroundWindow, MoveWindow, SW_RESTORE, ShowWindow};

pub mod finder;
pub fn get_input(prompt: &str) -> Result<String, Box<dyn Error>> {
  use std::io::{self, Write};

  print!("{} ", prompt);
  io::stdout().flush()?;

  let mut input = String::new();
  io::stdin().read_line(&mut input)?;
  Ok(input.trim().to_string())
}
pub fn get_cursor_position() -> Result<POINT, Box<dyn Error>> {
  let mut point = POINT { x: 0, y: 0 };
  unsafe {
    GetCursorPos(&mut point)?;
  }
  Ok(point)
}

pub fn get_monitor_info() -> Result<MONITORINFO, Box<dyn Error>> {
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

pub fn position_window(hwnd: HWND, x: i32, y: i32, width: i32, height: i32) -> Result<(), Box<dyn Error>> {
  unsafe {
    ShowWindow(hwnd, SW_RESTORE).ok()?;
    MoveWindow(hwnd, x, y, width, height, true)?;
  }
  Ok(())
}

pub fn validate_pos(start_pos: &POINT, work_area: &RECT) -> Result<(), Box<dyn Error>> {
  if start_pos.x < work_area.left
    || start_pos.x > work_area.right
    || start_pos.y < work_area.top
    || start_pos.y > work_area.bottom
  {
    return Err("Starting position is outside the screen work area".into());
  }
  Ok(())
}
