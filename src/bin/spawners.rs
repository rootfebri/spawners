use spawners::finder::{descendant_pids, hwnds_for_exe, hwnds_for_pids};
use spawners::{get_cursor_position, get_input, get_monitor_info, position_window, validate_pos};
use std::collections::HashSet;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;
use windows::Win32::System::Threading::Sleep;

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
  let mut children: Vec<Child> = Vec::with_capacity(count);
  for _ in 0..count {
    match Command::new(&program).spawn() {
      Ok(child) => {
        println!("Spawned instance of {program} (pid {})", child.id());
        children.push(child);
        thread::sleep(Duration::from_millis(300));
      }
      Err(e) => eprintln!("{e}"),
    }
  }

  _ = get_input("Press enter if all Programs have been spawned!");
  // Gather candidate PIDs: direct children plus any descendants (some launchers re-spawn UI in another PID)
  let root_pids: HashSet<u32> = children.iter().map(|c| c.id()).collect();
  // brief settle time before snapshotting
  thread::sleep(Duration::from_millis(1200));
  let mut all_pids = root_pids.clone();
  if let Ok(desc) = descendant_pids(&root_pids) {
    all_pids.extend(desc);
  }

  // Prefer PID-based hwnd discovery; fallback to exe-name if none found
  let mut handles = hwnds_for_pids(&all_pids)?;
  if handles.is_empty() {
    handles = hwnds_for_exe(program.as_ref())?;
  }
  // Get monitor information
  let monitor_info = get_monitor_info()?;
  let work_area = monitor_info.rcWork;

  // Get starting position from user
  println!("\nMove your mouse to the desired starting position and press Enter...");
  let _ = get_input("Press Enter when ready")?;
  let start_pos = get_cursor_position()?;

  validate_pos(&start_pos, &work_area)?;

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
