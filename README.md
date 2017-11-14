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
In the following example, the `example-daemon.sh` script daemonizes
an infinitely sleeping process and spits out its pid.
([source](../master/examples/example-daemon.rs))
```shell
$ ./example-daemon.sh
13204
$ grep PPid /proc/13204/status
PPid: 1
$ 
$ reaper ./example-daemon.sh
13216
$ stat /proc/13216
stat: cannot stat '/proc/13216': No such file or directory
$ 
$ RUST_LOG=reaper=info reaper ./example-daemon.sh
INFO:reaper: spawned child (pid=13222): ./example-daemon.sh
INFO:reaper: child terminated (pid=13222,exit=0), continuing to reap its orphans
INFO:reaper: sending SIGTERM to orphan (pid=13228)
INFO:reaper: 1 orphan(s) reaped
INFO:reaper: exiting with status=0
13228
```
