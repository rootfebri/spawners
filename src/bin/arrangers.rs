use spawners::finder::hwnds_for_exe;
use spawners::{get_cursor_position, get_input, get_monitor_info, position_window, validate_pos};
use std::env;
use std::io::{self, Write};
use windows::Win32::System::Threading::Sleep;

struct ArrangeConfig {
  program_name: String,
  max_h_stack: usize,
  max_v_stack: usize,
  horizontal_spacing: i32,
  vertical_spacing: i32,
}

fn get_flag_or_prompt(args: &[String], flag: &str, prompt: &str) -> String {
  let mut iter = args.iter();
  while let Some(arg) = iter.next() {
    if arg == flag {
      return iter.next().cloned().unwrap_or_else(|| prompt_user(prompt));
    }
  }
  prompt_user(prompt)
}

fn prompt_user(prompt: &str) -> String {
  print!("{}", prompt);
  io::stdout().flush().unwrap();
  let mut input = String::new();
  io::stdin().read_line(&mut input).unwrap();
  input.trim().to_string()
}

fn parse_or_prompt<T: std::str::FromStr>(input: String, prompt: &str) -> T {
  input.parse().unwrap_or_else(|_| {
    loop {
      let val = prompt_user(prompt);
      if let Ok(parsed) = val.parse() {
        break parsed;
      }
      println!("Invalid input, try again.");
    }
  })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args: Vec<String> = env::args().collect();

  let program_name = get_flag_or_prompt(&args, "--program", "Program name: ");

  let max_h_stack = parse_or_prompt(get_flag_or_prompt(&args, "--max-h", "Max H-Stack: "), "Max H-Stack: ");
  let max_v_stack = parse_or_prompt(get_flag_or_prompt(&args, "--max-v", "Max V-Stack: "), "Max V-Stack: ");
  let horizontal_spacing = parse_or_prompt(
    get_flag_or_prompt(&args, "--h-spacing", "Horizontal spacing: "),
    "Horizontal spacing: ",
  );
  let vertical_spacing = parse_or_prompt(
    get_flag_or_prompt(&args, "--v-spacing", "Vertical spacing: "),
    "Vertical spacing: ",
  );

  let config = ArrangeConfig {
    program_name: program_name.clone(),
    max_h_stack,
    max_v_stack,
    horizontal_spacing,
    vertical_spacing,
  };

  // Find existing windows for the specified program
  println!("Finding existing windows for '{}'...", program_name);
  let handles = hwnds_for_exe(program_name.as_ref())?;

  if handles.is_empty() {
    return Err(format!("No windows found for program '{}'", program_name).into());
  }

  println!("Found {} windows for '{}'", handles.len(), program_name);

  // Get monitor information
  let monitor_info = get_monitor_info()?;
  let work_area = monitor_info.rcWork;

  // Get starting position from mouse
  println!("\nMove your mouse to the desired starting position and press Enter...");
  let _ = get_input("Press Enter when ready")?;
  let start_pos = get_cursor_position()?;

  validate_pos(&start_pos, &work_area)?;

  // Calculate window positions based on spacing configuration
  let window_width = 308;
  let window_height = 265;

  let mut positions = Vec::new();
  for v in 0..config.max_v_stack {
    for h in 0..config.max_h_stack {
      if positions.len() >= handles.len() {
        break;
      }
      let x = start_pos.x + (h as i32 * (window_width + config.horizontal_spacing));
      let y = start_pos.y + (v as i32 * (window_height + config.vertical_spacing));
      positions.push((x, y));
    }
    if positions.len() >= handles.len() {
      break;
    }
  }

  // Position windows
  println!("\nArranging {} windows for '{}':", handles.len(), config.program_name);
  for ((i, hwnd), (x, y)) in handles.iter().enumerate().zip(positions) {
    let h = i % config.max_h_stack;
    let v = i / config.max_h_stack;
    match position_window(*hwnd, x, y, window_width, window_height) {
      Ok(_) => println!("Window [{}, {}] positioned at ({}, {})", h, v, x, y),
      Err(e) => eprintln!("Failed to position window [{}, {}]: {}", h, v, e),
    }
    unsafe {
      Sleep(500);
    }
  }

  println!("\nOperation completed! Arranged {} windows.", handles.len());
  Ok(())
}
