#!/bin/sh
set -o errexit
daemon_pid=$($REAPER $DAEMON)
if [ -e /proc/$daemon_pid ]; then
    echo 1>&2 "reaper failed, killing $daemon_pid manually..."
    kill $daemon_pid
    exit 1
fi
