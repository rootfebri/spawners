use spawners::get_input;

fn main() {
  let pid: u32 = get_input("PID:").unwrap().parse().unwrap();
  let names = spawners::finder::get_process_name_by_pid(pid).unwrap();
  println!("{names}");
}
