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
run "grep PPid /proc/$OUTPUT/status"
run ""
run "reaper ./example-daemon.sh"
run "stat /proc/$OUTPUT"
run ""
run "RUST_LOG=reaper=info reaper ./example-daemon.sh"
run ""
run "timeout --signal=SIGINT 2s ./lingering-parent-with-daemon.sh"
run "grep PPid /proc/$OUTPUT/status"
run ""
run "RUST_LOG=reaper=info timeout --signal=SIGINT 2s reaper ./lingering-parent-with-daemon.sh"
cat <<EOF
\`\`\`
EOF
