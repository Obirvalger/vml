use std::io::prelude::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::Result;

pub fn reply(message: &[u8], socket_path: &PathBuf) -> Result<Vec<u8>> {
    let socat = Command::new("socat")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .arg(socket_path)
        .spawn()?;

    socat.stdin.unwrap().write_all(message)?;
    let mut reply = Vec::new();
    socat.stdout.unwrap().read_to_end(&mut reply)?;
    Ok(reply)
}
