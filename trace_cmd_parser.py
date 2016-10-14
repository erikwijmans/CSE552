#!/usr/bin/env python2
import fileinput, sys, re

'./parser.py t_start t_stop name'


start = float(sys.argv[1])
stop = float(sys.argv[2])
thread_names = sys.argv[3:]

parser = re.compile("\s*(.*)-\d*\s*\[(\d*)\]\s.(\d*\.\d*):\ssched_switch:\s*(.*):.*==>\s*(.*):.*")

trace = [parser.match(l) for l in sys.stdin if parser.match(l) is not None]

for name in thread_names:
  time_in = 0.0
  total = 0
  for line in trace:
    n1, core, t, n2, n3 = line.group(1, 2, 3, 4, 5)
    time = float(t)

    if time < start or time > stop:
      continue

    if n1 == name and time_in != 0.0:
      total += time - time_in
    elif n3 == name:
      time_in = time

  print name, "{:.3f} ms".format(total * 1e3), "{:2.3f}%".format(total/(stop - start)*1e2), "\n"