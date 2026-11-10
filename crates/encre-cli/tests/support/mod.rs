use serde_json::Value;
use std::io::Write;
use std::process::{Command, Stdio};

/// What one run of the built `encre` binary did.
pub struct Run {
    pub code: i32,
    pub stdout: Vec<u8>,
    pub stderr: String,
}

impl Run {
    /// Stdout as text.
    pub fn text(&self) -> String {
        String::from_utf8(self.stdout.clone()).expect("UTF-8 stdout")
    }

    /// Stdout as one JSON document.
    pub fn json(&self) -> Value {
        serde_json::from_slice(&self.stdout).unwrap_or_else(|e| panic!("stdout is not JSON ({e}): {}", self.text()))
    }
}

/// Runs `encre args...` with `stdin`, and waits.
pub fn encre(args: &[&str], stdin: &str) -> Run {
    let mut child = Command::new(env!("CARGO_BIN_EXE_encre"))
        .args(args)
        .env_remove("NO_COLOR")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn encre");
    let _ = child.stdin.take().expect("stdin").write_all(stdin.as_bytes());
    let out = child.wait_with_output().expect("wait for encre");
    Run {
        code: out.status.code().expect("exit code"),
        stdout: out.stdout,
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}
