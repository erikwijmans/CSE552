extern crate scheduler;

fn usage() {
  println!("./l2 <scheduler> <rounds> <iterations> <number>+");
}

#[derive(Debug, PartialEq)]
enum SCHED {
  RR,
  FIFO,
  NORMAL,
  NONE

}

fn parse_int(to_parse: &str) -> i32 {
  let parsed = to_parse.parse::<i32>();
  if parsed.is_err() {
    println!("Could not parse {:?} as an i32", to_parse);
    std::process::exit(1);
  }

  return parsed.unwrap();
}


fn main() {
  let argv: Vec<_> = std::env::args().collect();
  if argv.len() <= 5 {
    usage();
    return;
  }

  let sched = match argv[1].as_ref() {
    "SCHED_RR" => SCHED::RR,
    "SCHED_FIFO" => SCHED::FIFO,
    "SCHED_NORMAL" => SCHED::NORMAL,
    _ => SCHED::NONE
  };

  if sched == SCHED::NONE {
    println!("Unknown scheduler type: {:?}", argv[1]);
    std::process::exit(1);
  }

  let rounds = parse_int(&argv[2]);
  let iters = parse_int(&argv[3]);

  let numbers: Vec<i32> = argv[4..].into_iter()
                                    .map(|n| parse_int(&n))
                                    .collect();

  for i in 0..3 {
    let cpu = scheduler::CpuSet::single(i);
    std::thread::spawn(move || {
      scheduler::set_self_affinity(cpu);
    })
  }
}