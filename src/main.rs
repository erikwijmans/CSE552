extern crate scheduler;
use std::sync::{Arc, Mutex, atomic};
use std::io::Write;

const NBARRARIERS: usize = 2;
const NPRIOS: usize = 2;
const NTHREADS_PER_CORE: usize = 4;
const NCORES: usize = 4;
const START_INDEX: usize = 0;

// Arrays of priority and nice values, which array is used is determined
// by what scheduler is choosen.  The 3rd value in both arrays is the
// nice/prio value of the spawning thread
//
const NICE_VALUES: [i32; NPRIOS + 1] = [-5, 5, -10];
const PRIO_VALUES: [i32; NPRIOS + 1] = [98, 90, 99];
const TYPE: [&'static str; 4] = ["High", "Low", "High'", "Low'"];

// Usage string
fn usage() -> &'static str {
  "./lab_2 <scheduler> <rounds> <iterations> <number>+"
}

// Struct that holds the data needed to implement a spin barrier
#[derive(Debug)]
struct SpinBarrier {
  nthreads: usize,
  gen: atomic::AtomicUsize,
  nspinning: atomic::AtomicUsize
}

// Implement methods on the SpinBarrier struct
impl SpinBarrier {
  // Static function that serves as the constructor
  fn new(nthreads: usize) -> SpinBarrier {
    SpinBarrier {
      nthreads: nthreads,
      gen: atomic::AtomicUsize::new(0),
      nspinning: atomic::AtomicUsize::new(0)
    }
  }

  // Member function to wait on the barrier
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

// Function to safely parse a string to an interger
fn parse_int(to_parse: std::string::String) -> i32 {
  match to_parse.parse::<i32>() {
    Ok(n) => n,
    _ => panic!("Could not parse {:?} into an int", to_parse),
  }
}

// Safely binds a thread to a CPU using sched.h wrappers provided
// by the scheduler namespace
fn bind_cpu(cpu_number: usize) {
  let cpu = scheduler::CpuSet::single(cpu_number);

  match scheduler::set_self_affinity(cpu) {
    Ok(_) => return,
    Err(code) => panic!("Got error when binding on CPU {:?}", code),
  };
}

// Indexs the prio/nice arrays based on the scheduler and then
// sets both the scheduler and prio/nice value.
fn set_sched(scheduler: scheduler::Policy, tid: usize) {
  let prio = match &scheduler {
    &scheduler::Policy::RoundRobin |
    &scheduler::Policy::Fifo => PRIO_VALUES[tid],
    &scheduler::Policy::Other => NICE_VALUES[tid],
    _ => panic!("Unknown scheduler {:?}", scheduler),
  };

  match scheduler::set_self_policy(scheduler, prio) {
    Ok(_) => return,
    Err(code) => panic!("Got error code {} while setting the scheduler", code),
  };
}

fn main() {
  let mut argv = std::env::args();
  // Throw away the program name
  argv.next().unwrap();

  // Check to make sure we have enough arguements
  if argv.len() <= 4 {
    panic!("{:?}", usage());
  }

  // Parse the first arguement (scheduler)
  let scheduler = match argv.next().unwrap().as_ref() {
    "SCHED_RR" => scheduler::Policy::RoundRobin,
    "SCHED_FIFO" => scheduler::Policy::Fifo,
    "SCHED_OTHER" | "SCHED_NORMAL" => scheduler::Policy::Other,
    ref sched @ _ => panic!("Unknown scheduler type: {:?}", sched),
  };

  // Parse the number of rounds and the number of iterations
  let rounds: i32 = parse_int(argv.next().unwrap());
  let iters: i32 = parse_int(argv.next().unwrap());

  // Parses the remaining arguements into a vector of numbers
  let nums: Vec<i32> = argv.map(|n| parse_int(n))
    .collect();

  // Arc = shared_ptr
  let numbers = Arc::new(nums);
  let index = Arc::new(Mutex::new(START_INDEX));
  let mut barriers = Vec::with_capacity(NBARRARIERS);
  for _ in 0..NBARRARIERS {
    barriers.push(Arc::new(SpinBarrier::new(NCORES * NTHREADS_PER_CORE / NBARRARIERS)));
  }

  let mut children = Vec::with_capacity(NCORES * NTHREADS_PER_CORE);
  set_sched(scheduler, 2);

  // println!("Starting to spawn children");
  // std::io::stdout().flush();

  for cpu_number in 0..NCORES {
    for t in 0..NTHREADS_PER_CORE {
      let t_number = t % NPRIOS;
      let barrier_index = t % NBARRARIERS;
      // Rust doesn't support method overloading,
      // so a builder with chaining is used instead
      let builder = std::thread::Builder::new().name(format!("{}{}", TYPE[t], cpu_number));

      // Sets up new Arc's that will be passed to the thread
      let (numbers, index, barrier) =
        (numbers.clone(), index.clone(), barriers[barrier_index].clone());
      children.push(builder.spawn(move || { // This is a lambda

          /*println!("Spawned child {:?}", cpu_number * NTHREADS_PER_CORE + t);
          std::io::stdout().flush();*/

          bind_cpu(cpu_number);
          set_sched(scheduler, t_number);

         /* println!("child {:?} waiting on the SpinBarrier",
                   cpu_number * NTHREADS_PER_CORE + t);
          std::io::stdout().flush();*/

          barrier.wait();

          /*println!("child {:?} through SpinBarrier",
                   cpu_number * NTHREADS_PER_CORE + t);
          std::io::stdout().flush();*/

          for _ in 0..rounds {
            let to_mult;
            {
              // Uses Lock Guard idiom
              let mut locked_index = index.lock().unwrap();

              to_mult = numbers[*locked_index];
              *locked_index = (*locked_index + 1) % numbers.len();
            }

            for _ in 0..iters {
              let x = to_mult * to_mult * to_mult;
            }
          }
        })
        .unwrap());

      // std::thread::sleep(std::time::Duration::from_millis(1000));
    }
  }

  // println!("I am done spawning my children");
  // std::io::stdout().flush();

  for child in children {
    let _ = child.join();
  }
}
