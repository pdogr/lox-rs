use std::env;
use std::ffi::OsStr;
use std::io::Write;
use std::process::{Command, ExitStatus, Stdio};

// Taken from BurntSuchi/cargo-benchcmp/tests/integration.rs
#[derive(Debug)]
pub struct CommandUnderTest {
    raw: Command,
    stdin: Vec<u8>,
    run: bool,
    stdout: String,
    stderr: String,
    binary_path: String,
}

impl CommandUnderTest {
    pub fn new(binary_name: String) -> CommandUnderTest {
        // To find the directory where the built binary is, we walk up the directory tree of the test binary until the
        // parent is "target/".
        let mut binary_path =
            env::current_exe().expect("need current binary path to find binary to test");
        loop {
            {
                let parent = binary_path.parent();
                if parent.is_none() {
                    panic!(
                        "Failed to locate binary path from original path: {:?}",
                        env::current_exe()
                    );
                }
                let parent = parent.unwrap();
                if parent.is_dir() && parent.file_name().unwrap() == "target" {
                    break;
                }
            }
            binary_path.pop();
        }

        binary_path.push(if cfg!(target_os = "windows") {
            format!("{}.exe", binary_name)
        } else {
            binary_name
        });

        let mut cmd = Command::new(binary_path.clone());
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        CommandUnderTest {
            raw: cmd,
            run: false,
            stdin: Vec::new(),
            stdout: String::new(),
            stderr: String::new(),
            binary_path: binary_path
                .into_os_string()
                .into_string()
                .expect("Error: Cannot convert PathBuf to String."),
        }
    }

    pub fn clone_cmd(&self) -> Self {
        let mut cmd = Command::new(self.binary_path.clone());
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        Self {
            raw: cmd,
            run: false,
            stdin: Vec::new(),
            stdout: String::new(),
            stderr: String::new(),
            binary_path: self.binary_path.clone(),
        }
    }

    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        self.raw.arg(arg);
        self
    }

    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.raw.args(args);
        self
    }

    pub fn run(&mut self) -> ExitStatus {
        let mut child = self.raw.spawn().expect("failed to run command");

        if !self.stdin.is_empty() {
            let stdin = child.stdin.as_mut().expect("failed to open stdin");
            stdin
                .write_all(&self.stdin)
                .expect("failed to write to stdin")
        }

        let output = child
            .wait_with_output()
            .expect("failed waiting for command to complete");
        self.stdout = String::from_utf8(output.stdout).unwrap();
        self.stderr = String::from_utf8(output.stderr).unwrap();
        self.run = true;
        output.status
    }
}
