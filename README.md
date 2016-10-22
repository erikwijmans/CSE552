# Lab2, Linux Scheduler Profiling

## Contact information
[Erik Wijmans](mailto:erikwijmans@wustl.edu)
[Ethan Vaughan](mailto:evaughan@wustl.edu)
[Sam Frank](mailto:sjfrank@wustl.edu)

## Sources

For this lab, we used (and modified) a rust wrapper to the utilites found in sched.h called [scheduler](https://crates.io/crates/scheduler/0.1.3)

## Design

We used rust as the programming langauge for this lab.  Rust was choosen for several reasons:
+ Systems programming langauge with a wonderful concurrency model that prides itself on safe concurrency with extemely low cost abstractions.
+ Lots of cool features and syntactic surgar.
+ Very interested in learnig it.

Rust's concurrency model was the driving factor for how we structured our code.

## Analysis

A full anaylsis of this lab can be found in writeup.pdf

## Paramters
Rounds: 1000

Iterations: 10000

## Questions
Sched_FIFO with four threads does some very strange things and we would very much like to understand exactly why it does that and how to fix it.


## Time Spent

5 hours coding/debugging the test programming and the parser srcipt

10 hours doing the writeup and analysis
