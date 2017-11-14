#!/bin/bash

if [[ -v OUT ]]; then
    exec 1<&-
    exec 2<&-
    exec 1<>$OUT
    exec 2>&1
fi

run() {
    echo "$ $@"
    OUTPUT=$(eval $@)
    if [ -n "$OUTPUT" ]; then
        echo $OUTPUT
    fi
}

cat <<EOF
Don't fear the reaper
=====================
Or more appropriately, don't fear the subreaper:
[prctl(PR_SET_CHILD_SUBREAPER,..)](http://man7.org/linux/man-pages/man2/prctl.2.html)

Imagine that a program spawns some resource intensive daemons,
and when your program ends you want the daemons to stop as well?
Using the Linux specific option to \`prctl\` and
[/proc/pid/stat](http://man7.org/linux/man-pages/man5/proc.5.html) one
can reap and terminate the remaining orphans after your program halts.

Example usage
-------------
In the following example, the \`example-daemon.sh\` script daemonizes
an infinitely sleeping process and spits out its pid.
([source](../master/examples/example-daemon.rs))
\`\`\`shell
EOF
run "./example-daemon.sh"
PID=$OUTPUT
run "grep PPid /proc/$PID/status"
run ""
run "reaper ./example-daemon.sh"
PID=$OUTPUT
run "stat /proc/$PID"
run ""
run "RUST_LOG=reaper=info reaper ./example-daemon.sh"
cat <<EOF
\`\`\`
EOF
