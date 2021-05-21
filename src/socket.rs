use std::io::prelude::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use crate::{Error, Result};

pub fn reply(message: &[u8], socket_path: &Path) -> Result<Vec<u8>> {
    let socat = Command::new("socat")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .arg(socket_path)
        .spawn()
        .map_err(|e| Error::executable("socat", &e.to_string()))?;

    socat.stdin.unwrap().write_all(message)?;
    let mut reply = Vec::new();
    socat.stdout.unwrap().read_to_end(&mut reply)?;
    Ok(reply)
}
