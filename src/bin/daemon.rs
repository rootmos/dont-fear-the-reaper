#[macro_use]

extern crate log;
use log::*;
extern crate env_logger;

extern crate nix;
use nix::unistd::{fork, ForkResult, getppid, close};
use std::{time, thread};

extern crate libc;
use libc::STDOUT_FILENO;

fn main() {
    env_logger::init().unwrap();

    match fork() {
        Ok(ForkResult::Parent {child}) => {
            debug!("forked daemon (pid={})", child);
            println!("{}", child);
            debug!("stopping parent");
        }
        Ok(ForkResult::Child) => {
            close(STDOUT_FILENO).unwrap();
            loop {
                debug!("daemon running: ppid={}", getppid());
                thread::sleep(time::Duration::from_secs(1));
            }
        }
        Err(e) => panic!("fork failed: {}", e)
    }
}
