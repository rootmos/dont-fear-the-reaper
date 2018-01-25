#[macro_use]

extern crate log;
use log::*;
extern crate env_logger;

extern crate nix;
use nix::unistd::{fork, ForkResult, getppid, close, getpid, setpgid};
use std::{time, thread, env};

extern crate libc;
use libc::STDOUT_FILENO;

fn main() {
    env_logger::init().unwrap();

    match fork() {
        Ok(ForkResult::Parent {child}) => {
            debug!("forked daemon (pid={})", child);
            println!("{}", child);

            if env::var("LINGERING_PARENT").is_ok() {
                loop {
                    debug!("parent running: pid={}", getpid());
                    thread::sleep(time::Duration::from_secs(1));
                }
            } else {
                debug!("stopping parent");
            }
        }
        Ok(ForkResult::Child) => {
            //setpgid(getpid(), getpid()).unwrap();
            close(STDOUT_FILENO).unwrap();
            loop {
                debug!("daemon running: ppid={}", getppid());
                thread::sleep(time::Duration::from_secs(1));
            }
        }
        Err(e) => panic!("fork failed: {}", e)
    }
}
