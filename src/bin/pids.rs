use spawners::get_input;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
  let name = get_input("App name:")?;
  let apps = spawners::finder::pids_by_exe_name(&name)?;
  println!("{apps:#?}");
  Ok(())
}
