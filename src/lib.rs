pub mod finder;
pub fn get_input(prompt: &str) -> anyhow::Result<String, Box<dyn std::error::Error>> {
  use std::io::{self, Write};

  print!("{} ", prompt);
  io::stdout().flush()?;

  let mut input = String::new();
  io::stdin().read_line(&mut input)?;
  Ok(input.trim().to_string())
}
