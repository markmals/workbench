//! Running the programs `workbench` relies on — `git`, `gh`, `bd` — and reading what they report.

use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// A program that ran to completion, whatever its exit status.
#[derive(Clone, Debug)]
pub struct Run {
    pub status: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl Run {
    pub fn success(&self) -> bool {
        self.status == Some(0)
    }
}

/// A program that did not run to completion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunError {
    /// The program is not installed or not on PATH.
    Missing,
    /// The program ran past its time limit and was killed.
    TimedOut,
    /// The program could not be started for another reason.
    Spawn(String),
}

/// One program invocation. Environment entries apply on top of the inherited environment.
pub struct Program<'a> {
    program: &'a str,
    args: Vec<String>,
    cwd: &'a Path,
    timeout: Option<Duration>,
    env: Vec<(String, String)>,
}

impl<'a> Program<'a> {
    pub fn new(program: &'a str, cwd: &'a Path) -> Program<'a> {
        Program {
            program,
            args: Vec::new(),
            cwd,
            timeout: None,
            env: Vec::new(),
        }
    }

    pub fn arg(mut self, arg: impl Into<String>) -> Program<'a> {
        self.args.push(arg.into());
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Program<'a>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Program<'a> {
        self.timeout = Some(timeout);
        self
    }

    /// Gives the program until `deadline`, or a millisecond when the deadline has passed.
    pub fn deadline(self, deadline: Instant) -> Program<'a> {
        let remaining = deadline.saturating_duration_since(Instant::now());
        self.timeout(remaining.max(Duration::from_millis(1)))
    }

    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Program<'a> {
        self.env.push((key.into(), value.into()));
        self
    }

    pub fn run(self) -> Result<Run, RunError> {
        let mut command = Command::new(self.program);
        command
            .args(&self.args)
            .current_dir(self.cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (key, value) in &self.env {
            command.env(key, value);
        }

        let mut child = command.spawn().map_err(|error| match error.kind() {
            std::io::ErrorKind::NotFound => RunError::Missing,
            _ => RunError::Spawn(error.to_string()),
        })?;

        let stdout = drain(child.stdout.take());
        let stderr = drain(child.stderr.take());
        let started = Instant::now();
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) => {}
                Err(error) => return Err(RunError::Spawn(error.to_string())),
            }
            if self.timeout.is_some_and(|limit| started.elapsed() >= limit) {
                let _ = child.kill();
                let _ = child.wait();
                return Err(RunError::TimedOut);
            }
            thread::sleep(Duration::from_millis(5));
        };

        Ok(Run {
            status: status.code(),
            stdout: stdout.join().unwrap_or_default(),
            stderr: stderr.join().unwrap_or_default(),
        })
    }
}

fn drain<R: Read + Send + 'static>(stream: Option<R>) -> thread::JoinHandle<String> {
    thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(mut stream) = stream {
            let _ = stream.read_to_end(&mut bytes);
        }
        String::from_utf8_lossy(&bytes).into_owned()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_a_missing_program() {
        let result = Program::new("workbench-test-no-such-program", Path::new(".")).run();
        assert_eq!(result.unwrap_err(), RunError::Missing);
    }

    #[test]
    fn kills_a_program_past_its_time_limit() {
        let started = Instant::now();
        let result = Program::new("sleep", Path::new("."))
            .arg("5")
            .timeout(Duration::from_millis(100))
            .run();
        assert_eq!(result.unwrap_err(), RunError::TimedOut);
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    #[test]
    fn captures_output_and_status() {
        let run = Program::new("sh", Path::new("."))
            .args(["-c", "echo out; echo err >&2; exit 3"])
            .run()
            .unwrap();
        assert_eq!(run.status, Some(3));
        assert_eq!(run.stdout, "out\n");
        assert_eq!(run.stderr, "err\n");
    }
}
