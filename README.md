Don't fear the reaper
=====================
Or more appropriately, don't fear the subreaper:
[prctl(PR_SET_CHILD_SUBREAPER,..)](http://man7.org/linux/man-pages/man2/prctl.2.html)

Imagine that a program spawns some resource intensive daemons,
and when your program ends you want the daemons to stop as well?
Using the Linux specific option to `prctl` and
[/proc/pid/stat](http://man7.org/linux/man-pages/man5/proc.5.html) one
can reap and terminate the remaining orphans after your program halts.

Example usage
-------------
In the following example, the `example-daemon` program just spits out
its pid and becomes a daemon.
(The `RUST_LOG` variables and `cargo run` are just for visualization and
convenience.)
```shell
$ RUST_LOG=example_daemon=debug \
    cargo run --example example-daemon --quiet 2>/tmp/log
3312
$ ! timeout 5s tail --pid=3312 -f /tmp/log && echo "...just keeps going!"
DEBUG:example_daemon: daemon running: ppid=1
DEBUG:example_daemon: daemon running: ppid=1
DEBUG:example_daemon: daemon running: ppid=1
DEBUG:example_daemon: daemon running: ppid=1
DEBUG:example_daemon: daemon running: ppid=1
DEBUG:example_daemon: daemon running: ppid=1
DEBUG:example_daemon: daemon running: ppid=1
DEBUG:example_daemon: daemon running: ppid=1
DEBUG:example_daemon: daemon running: ppid=1
DEBUG:example_daemon: daemon running: ppid=1
DEBUG:example_daemon: daemon running: ppid=1
DEBUG:example_daemon: daemon running: ppid=1
DEBUG:example_daemon: daemon running: ppid=1
DEBUG:example_daemon: daemon running: ppid=1
DEBUG:example_daemon: daemon running: ppid=1
...just keeps going!
$ kill 3312
$
$ RUST_LOG=reaper=debug,example_daemon=debug \
    reaper cargo run --example example-daemon --quiet 2>/tmp/log
5473
$ kill 5473
$ cat /tmp/log
INFO:reaper: spawned child (pid=5467): cargo
DEBUG:example_daemon: forked daemon (pid=5473)
DEBUG:example_daemon: stopping parent
DEBUG:example_daemon: daemon running: ppid=5467
INFO:reaper: child exited (pid=5467,exit=0)
INFO:reaper: child terminated (pid=5467,exit=0), continuing to reap its orphans
DEBUG:reaper: sending SIGTERM to orphan (pid=5473)
DEBUG:reaper: final orphan states: [Carcas(Carcas { pid: Pid(5473), status: None, signal: Some(SIGTERM) })]
$
```
