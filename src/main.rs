extern crate scheduler;
use std::sync::{Arc, Mutex, atomic};

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

#[derive(Debug, Clone, Copy)]
enum Sched {
  RR,
  FIFO,
  OTHER
}

fn usage() {
  println!("./l2 <scheduler> <rounds> <iterations> <number>+");
}

static NTHREADS: usize = 2;
static NCORES: usize = 4;

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

fn set_sched(scheduler: Sched, tid: usize) {
  let res = match scheduler {
    Sched::RR => scheduler::set_self_policy(scheduler::Policy::RoundRobin, 0),
    Sched::FIFO => scheduler::set_self_policy(scheduler::Policy::Fifo, 0),
    Sched::OTHER => scheduler::set_self_policy(scheduler::Policy::Other, 0),
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
    "SCHED_RR" => Sched::RR,
    "SCHED_FIFO" => Sched::FIFO,
    "SCHED_OTHER" | "SCHED_NORMAL" => Sched::OTHER,
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

  let numbers = Arc::new(nums);
  let index = Arc::new(Mutex::new(0));
  let barrier = Arc::new(SpinBarrier::new(NCORES * NTHREADS));

  let mut children = Vec::with_capacity(NCORES * NTHREADS);
  for cpu_number in 0..NCORES {
    for tid in 0..NTHREADS {
      let numbers = numbers.clone();
      let index = index.clone();
      let barrier = barrier.clone();

      children.push(std::thread::spawn(move || {
        bind_cpu(cpu_number);
        set_sched(scheduler, tid);
        barrier.wait();

        for _ in 0..rounds {
          let mult;
          {
            let mut index = index.lock().unwrap();

            mult = numbers[*index];
            *index = if *index + 1 == numbers.len() {
              0
            } else {
              *index + 1
            };
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
