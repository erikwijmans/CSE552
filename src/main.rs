extern crate scheduler;
use std::sync::{Arc, Mutex, atomic};

const NTHREADS_PER_CORE: usize = 2;
const NCORES: usize = 4;
const START_INDEX: usize = 0;
const NICE_VALUES: [i32; NTHREADS_PER_CORE] = [-5, 5];
const PRIO_VALUES: [i32; NTHREADS_PER_CORE] = [40, 20];
const TYPE: [&'static str; NTHREADS_PER_CORE] = ["High", "Low"];

#[derive(Debug)]
struct SpinBarrier {
  nthreads: usize,
  gen: atomic::AtomicUsize,
  nspinning: atomic::AtomicUsize
}

impl SpinBarrier {
  fn new(nthreads: usize) -> SpinBarrier {
    SpinBarrier {
      nthreads: nthreads,
      gen: atomic::AtomicUsize::new(0),
      nspinning: atomic::AtomicUsize::new(0)
    }
  }

  fn wait(&self) {
    if self.nspinning.fetch_add(1, atomic::Ordering::Relaxed) == self.nthreads - 1 {
      self.gen.fetch_add(1, atomic::Ordering::Relaxed);
      self.nspinning.store(0, atomic::Ordering::Relaxed);

    } else {
      let mygen = self.gen.load(atomic::Ordering::Relaxed);
      while self.gen.load(atomic::Ordering::Relaxed) == mygen {}
    }
  }
}

fn usage() {
  println!("./lab_2 <scheduler> <rounds> <iterations> <number>+");
}

fn parse_int(to_parse: std::string::String) -> i32 {
  match to_parse.parse::<i32>() {
    Ok(n) => n,
    _ => {
      println!("Could not parse {:?} into an int", to_parse);
      std::process::exit(1);
    }
  }
}

fn bind_cpu(cpu_number: usize) {
  let cpu = scheduler::CpuSet::single(cpu_number);

  match scheduler::set_self_affinity(cpu) {
    Ok(_) => return,
    Err(code) => {
      println!("Error code {} while trying to bind on a cpu", code);
      std::process::exit(1);
    }
  };
}

fn set_sched(scheduler: scheduler::Policy, tid: usize) {
  let prio = match &scheduler {
    &scheduler::Policy::RoundRobin |
    &scheduler::Policy::Fifo => PRIO_VALUES[tid],
    &scheduler::Policy::Other => NICE_VALUES[tid],
    _ => {
      println!("Unknown scheduler {:?}", scheduler);
      std::process::exit(1);
    }
  };

  match scheduler::set_self_policy(scheduler, prio) {
    Ok(_) => return,
    Err(code) => {
      println!("Got error code {} while setting the scheduler", code);
      std::process::exit(1);
    }
  };
}

fn main() {
  let mut argv = std::env::args();

  if argv.len() <= 5 || argv.next().unwrap() != "lab_2" {
    usage();
    std::process::exit(1);
  }

  let scheduler = match argv.next().unwrap().as_ref() {
    "SCHED_RR" => scheduler::Policy::RoundRobin,
    "SCHED_FIFO" => scheduler::Policy::Fifo,
    "SCHED_OTHER" | "SCHED_NORMAL" => scheduler::Policy::Other,
    ref sched @ _ => {
      println!("Unknown scheduler type: {:?}", sched);
      std::process::exit(1);
    }
  };

  let rounds: i32 = parse_int(argv.next().unwrap());
  let iters: i32 = parse_int(argv.next().unwrap());

  let nums: Vec<i32> = argv.map(|n| parse_int(n))
    .collect();

  // Arc = shared_ptr
  let numbers = Arc::new(nums);
  let index = Arc::new(Mutex::new(START_INDEX));
  let barrier = Arc::new(SpinBarrier::new(NCORES * NTHREADS_PER_CORE));

  let mut children = Vec::with_capacity(NCORES * NTHREADS_PER_CORE);
  for cpu_number in 0..NCORES {
    for tid in 0..NTHREADS_PER_CORE {
      let numbers = numbers.clone();
      let index = index.clone();
      let barrier = barrier.clone();

      // Rust doesn't support method overloading,
      // so a builder with chaining is used instead
      children.push(std::thread::Builder::new()
        .name(format!("{}{}", TYPE[tid], cpu_number))
        // This is a lambda
        .spawn(move || {
          bind_cpu(cpu_number);
          set_sched(scheduler, tid);
          barrier.wait();

          for _ in 0..rounds {
            let mult;
            {
              // Uses Lock Guard idiom
              let mut locked_index = index.lock().unwrap();

              mult = numbers[*locked_index];
              *locked_index = (*locked_index + 1) % numbers.len();
            }

            for _ in 0..iters {
              let x = mult * mult * mult;
            }
          }
        })
        .unwrap());

      // std::thread::sleep(std::time::Duration::from_millis(1000));
    }
  }

  for child in children {
    let _ = child.join();
  }
}
