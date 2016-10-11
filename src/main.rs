extern crate scheduler;
use std::sync::{Arc, Mutex, atomic};

static NTHREADS_PER_CORE: usize = 2;
static NCORES: usize = 4;
static START_INDEX: usize = 0;
static NICE_VALUES: [i32; 2] = [-5, 5];
static PRIO_VALUES: [i32; 2] = [20, 40];

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
  println!("./l2 <scheduler> <rounds> <iterations> <number>+");
}

fn parse_int(to_parse: &str) -> i32 {
  let parsed = to_parse.parse::<i32>();
  if parsed.is_err() {
    println!("Could not parse {:?} as an i32", to_parse);
    std::process::exit(1);
  }

  parsed.unwrap()
}

fn bind_cpu(cpu_number: usize) {
  let cpu = scheduler::CpuSet::single(cpu_number);
  let res = scheduler::set_self_affinity(cpu);
  if res.is_err() {
    println!("Got error in bind_cpu {:?}", res);
  }
}

fn set_sched(scheduler: scheduler::Policy, tid: usize) {
  let res = match scheduler {
    scheduler::Policy::RoundRobin |
    scheduler::Policy::Fifo => scheduler::set_self_policy(scheduler, PRIO_VALUES[tid]),
    scheduler::Policy::Other => scheduler::set_self_policy(scheduler, NICE_VALUES[tid]),
    _ => {
      println!("Unknown scheduler {:?}", scheduler);
      std::process::exit(1);
    }
  };

  if res.is_err() {
    println!("Got error in set_sched {:?}", res);
    std::process::exit(1);
  }
}

fn main() {
  let argv: Vec<_> = std::env::args().collect();
  if argv.len() <= 5 {
    usage();
    std::process::exit(1);
  }

  let scheduler = match argv[1].as_ref() {
    "SCHED_RR" => scheduler::Policy::RoundRobin,
    "SCHED_FIFO" => scheduler::Policy::Fifo,
    "SCHED_OTHER" | "SCHED_NORMAL" => scheduler::Policy::Other,
    _ => {
      println!("Unknown scheduler type: {:?}", argv[1]);
      std::process::exit(1);
    }
  };

  let rounds: i32 = parse_int(&argv[2]);
  let iters: i32 = parse_int(&argv[3]);

  let nums: Vec<i32> = argv[4..]
    .into_iter()
    .map(|n| parse_int(&n))
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

      // This is a lambda
      children.push(std::thread::spawn(move || {
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
      }));
    }
  }

  for child in children {
    let _ = child.join();
  }
}
