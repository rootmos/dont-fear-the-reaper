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
21715
$ grep PPid /proc/21715/status
PPid: 1
$ 
$ reaper ./example-daemon.sh
21727
$ stat /proc/21727
stat: cannot stat '/proc/21727': No such file or directory
$ 
$ RUST_LOG=reaper=info reaper ./example-daemon.sh
INFO:reaper: spawned child (pid=21733): ./example-daemon.sh
INFO:reaper: child terminated (pid=21733,exit=0), continuing to reap its orphans
INFO:reaper: sending SIGTERM to orphan (pid=21739)
INFO:reaper: 1 orphan(s) reaped
INFO:reaper: exiting with status=0
21739
$ 
$ timeout --signal=SIGINT 2s ./lingering-parent-with-daemon.sh
21749
$ grep PPid /proc/21749/status
PPid: 1
$ 
$ RUST_LOG=reaper=info timeout --signal=SIGINT 2s reaper ./lingering-parent-with-daemon.sh
INFO:reaper: spawned child (pid=21779): ./lingering-parent-with-daemon.sh
INFO:reaper: child terminated (pid=21779,sig=SIGINT), continuing to reap its orphans
INFO:reaper: sending SIGTERM to orphan (pid=21785)
INFO:reaper: 1 orphan(s) reaped
INFO:reaper: child received signal SIGINT
21785
```
