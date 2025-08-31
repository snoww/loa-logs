use std::{ffi::OsStr, os::windows::process::CommandExt};

pub struct Command;

impl Command {
    const DETACHED_PROCESS: u32 = 0x00000008;

    pub fn new<S: AsRef<OsStr>>(program: S) -> std::process::Command {
        let mut command = std::process::Command::new(program);
        command.creation_flags(Command::DETACHED_PROCESS);
        command
    }
}
