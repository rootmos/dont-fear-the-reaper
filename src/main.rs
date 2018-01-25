use std::time::{Instant, Duration};
use std::env;
use std::process::{Command, exit};
use std::fmt;
use std::fs::{read_dir, File};
use std::io::Read;
use std::collections::HashMap;

extern crate log;
use log::*;

extern crate env_logger;

extern crate nix;
use nix::sys::signal::kill;
use nix::sys::wait::{WaitStatus, waitpid, WNOHANG};
use nix::unistd::{Pid, getpid};
use nix::sys::signal::Signal;

extern crate libc;
use libc::{prctl, PR_SET_CHILD_SUBREAPER};

extern crate signal;
use signal::trap::Trap;
use signal::Signal::*;

#[derive(Clone, Debug)]
pub struct Carcass {
    pid: Pid,
    status: Option<i8>,
    signal: Option<Signal>,
}

impl fmt::Display for Carcass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match (self.status, self.signal) {
            (Some(st), None) => write!(f, "(pid={},exit={})", self.pid, st),
            (None, Some(sig)) => write!(f, "(pid={},sig={:?})", self.pid, sig),
            _ => unreachable!(),
        }
    }
}

fn reap() -> Option<Carcass> {
    match waitpid(None, Some(WNOHANG)).unwrap() {
        WaitStatus::Exited(pid, st) =>
            Some(Carcass { pid, status: Some(st), signal: None }),
        WaitStatus::Signaled(pid, sig, _) =>
            Some(Carcass { pid, status: None, signal: Some(sig) }),
        WaitStatus::StillAlive =>
            None,
        ws => {
            debug!("uninterpreted waitpid status: {:?}", ws);
            None
        }
    }
}

fn wait_for_child(trap: &mut Trap, child: Pid) -> Carcass {
    loop {
        match trap.next() {
            Some(SIGCHLD) => {
                if let Some(carcass) = reap() {
                    if carcass.pid == child {
                        return carcass;
                    } else {
                        debug!("reaped {}", carcass);
                    }
                }
            }
            Some(SIGINT) => {
                debug!("sending SIGINT to child (pid={})", child);
                match kill(child, Some(SIGINT)) {
                    Ok(()) => (),
                    Err(e) =>
                        warn!(
                            "unable to send SIGINT to child (pid={}): {}",
                            child, e),
                }
            }
            Some(SIGTERM) => {
                debug!("sending SIGTERM to child (pid={})", child);
                match kill(child, Some(SIGTERM)) {
                    Ok(()) => (),
                    Err(e) =>
                        warn!(
                            "unable to send SIGTERM to child (pid={}): {}",
                            child, e),
                }
            }
            x => panic!("unexpected trap {:?}", x)
        }
    }
}

fn list_children(parent: Pid) -> Vec<Pid> {
    read_dir("/proc").expect("unable to list /proc")
        .filter_map(|rde| {
            rde.ok().and_then(|de| {
                de.file_name().to_str()
                    .and_then(|fname| str::parse(fname).ok())
                    .map(|p| (de, Pid::from_raw(p)))
            })
        })
        .filter_map(|(de, pid)| {
            let mut path_buf = de.path();
            path_buf.push("stat");

            let mut s = String::new();
            let path = path_buf.as_path();
            match File::open(path).and_then(|mut f| f.read_to_string(&mut s)) {
                Ok(_) => {
                    if let Some(r) = s.split_whitespace().nth(3) {
                        match str::parse(r) {
                            Ok(p) => Some((pid, Pid::from_raw(p))),
                            _ => {
                                warn!("unable to interpret field 4 in {:?}",
                                      path);
                                None
                            }
                        }
                    } else {
                        warn!("unable to interpret {:?}", path);
                        None
                    }
                }
                Err(e) => {
                    warn!("unable to read {:?}: {}", path, e);
                    None
                }
            }
        })
        .filter_map(|(pid, ppid)|
            if ppid == parent { Some(pid) } else { None })
        .collect()
}

#[derive(Clone, Debug)]
enum OrphanState {
    BlissfulIgnorance(Pid),
    HasBeenSentSIGTERM(Pid),
    HasBeenSentSIGKILL(Pid, Instant),
    Errored(Pid, nix::Error),
    Carcass(Carcass),
}

fn transition_orphan(os: &mut OrphanState) {
    match *os {
        OrphanState::BlissfulIgnorance(pid) => {
            info!("sending SIGTERM to orphan (pid={})", pid);
            *os = match kill(pid, Some(SIGTERM)) {
                Ok(()) => OrphanState::HasBeenSentSIGTERM(pid),
                Err(e) => {
                    warn!(
                        "unable to send SIGTERM to orphan (pid={}): {}",
                        pid, e);
                    OrphanState::Errored(pid, e)
                }
            }
        }
        OrphanState::HasBeenSentSIGTERM(pid) => {
            info!("sending SIGKILL to orphan (pid={})", pid);
            *os = match kill(pid, Some(SIGKILL)) {
                Ok(()) => OrphanState::HasBeenSentSIGKILL(pid, Instant::now()),
                Err(e) => {
                    warn!(
                        "unable to send SIGKILL to orphan (pid={}): {}",
                        pid, e);
                    OrphanState::Errored(pid, e)
                }
            }
        }
        OrphanState::HasBeenSentSIGKILL(pid, i) => {
            warn!("orphan ({}) lingering (since {}s) after SIGKILL",
                  pid, i.elapsed().as_secs());
        }
        OrphanState::Carcass(_) => (),
        OrphanState::Errored(_, _) => (),
    }
}

fn in_final_state(os: &OrphanState) -> bool {
    match *os {
        OrphanState::BlissfulIgnorance(..) => false,
        OrphanState::HasBeenSentSIGTERM(..) => false,
        OrphanState::HasBeenSentSIGKILL(..) => false,
        OrphanState::Errored(..) | OrphanState::Carcass(..) => true,
    }
}

fn main() {
    env_logger::init().unwrap();

    unsafe { assert_eq!(prctl(PR_SET_CHILD_SUBREAPER, 1), 0); }

    let mut cmdline = env::args().skip(1);

    let cmd = cmdline.next().expect("specify command");
    let child = Command::new(&cmd)
        .args(cmdline)
        .spawn()
        .expect(&format!("unable to spawn {}", cmd));

    let child_pid = Pid::from_raw(child.id() as i32);
    info!("spawned child (pid={}): {}", child_pid, cmd);

    let trap = &mut Trap::trap(&[SIGCHLD, SIGINT, SIGTERM]);

    let child_carcass = wait_for_child(trap, child_pid);
    info!("child terminated {}, continuing to reap its orphans", child_carcass);

    let mut oss = HashMap::new();
    let pid = getpid();
    for p in list_children(pid) {
        let _ = oss.insert(p, OrphanState::BlissfulIgnorance(p));
    }

    let done = |oss: &HashMap<Pid, OrphanState>|
        oss.values().all(in_final_state);

    while !done(&oss) {
        oss.values_mut().for_each(transition_orphan);

        let deadline = Instant::now() + Duration::from_secs(1);
        while let Some(sig) = trap.wait(deadline) {
            match sig {
                SIGCHLD => {
                    if let Some(c) = reap() {
                        let _ = oss.insert(c.pid, OrphanState::Carcass(c));
                    }
                    if done(&oss) { break }
                },
                SIGINT => info!("ignoring SIGINT while reaping orphans"),
                SIGTERM => info!("ignoring SIGTERM while reaping orphans"),
                _ => unimplemented!(),
            }
        }

        for p in list_children(pid) {
            if !oss.contains_key(&p) {
                let _ = oss.insert(p, OrphanState::BlissfulIgnorance(p));
            }
        }
    }

    info!("{} orphan(s) reaped", oss.len());
    debug!("final orphan states: {:?}", oss.values());

    match child_carcass {
        Carcass { status: Some(st), .. } => {
            info!("exiting with status={}", st);
            exit(st as i32)
        }
        Carcass { signal: Some(sig), .. } => {
            info!("child received signal {:?}", sig);
            exit(1)
        }
        _ => unreachable!(),
    }
}
