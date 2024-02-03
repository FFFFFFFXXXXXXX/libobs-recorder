use std::{
    io::{self, BufRead, BufReader, BufWriter, StdinLock, StdoutLock, Write},
    path::{Component, Path},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    time::Duration,
};

use intprocess_recorder::settings::{Encoder, RecorderSettings};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum IpcCommand {
    Init {
        libobs_data_path: Option<String>,
        plugin_bin_path: Option<String>,
        plugin_data_path: Option<String>,
    },
    Configure(RecorderSettings),
    Encoders,
    StartRecording,
    StopRecording,
    IsRecording,
    Shutdown,
    Exit,
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum IpcResponse {
    Ok,
    Encoders { available: Vec<Encoder>, selected: Encoder },
    Recording(bool),
    Err(String),
}

#[derive(Debug)]
pub struct IpcLinkMaster {
    tx: BufWriter<ChildStdin>,
    rx: BufReader<ChildStdout>,
    buffer: String,
    child_process: Child,
    logging_enabled: bool,
}

impl IpcLinkMaster {
    pub fn new(executable: impl AsRef<Path>, enable_logging: bool) -> io::Result<Self> {
        let executable = executable.as_ref().canonicalize()?;

        let mut child_process = Command::new(executable.as_os_str())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .current_dir(executable.parent().unwrap_or_else(|| Path::new(&Component::RootDir)))
            .spawn()?;

        Ok(Self {
            tx: BufWriter::new(child_process.stdin.take().unwrap()),
            rx: BufReader::new(child_process.stdout.take().unwrap()),
            buffer: String::with_capacity(512),
            child_process,
            logging_enabled: enable_logging,
        })
    }

    pub fn send(&mut self, cmd: IpcCommand) -> IpcResponse {
        _ = serde_json::to_writer(&mut self.tx, &cmd);
        _ = self.tx.write(&[b'\n']);
        _ = self.tx.flush();

        let logging = self.logging_enabled;
        loop {
            let Ok(line) = self.read_line() else {
                return IpcResponse::Err("failed to read from recorder".into());
            };
            match serde_json::from_str::<IpcResponse>(line) {
                Ok(response) => return response,
                Err(_) if logging => print!("ipc_link: {line}"),
                Err(_) => { /* do nothing */ }
            }
        }
    }

    pub fn drain_logs(&mut self) {
        if !self.logging_enabled {
            return;
        }

        while let Ok(log) = self.read_line() {
            print!("ipc_link: {log}");
        }
    }

    fn read_line(&mut self) -> Result<&str, ()> {
        self.buffer.clear();

        match self.rx.read_line(&mut self.buffer) {
            Ok(0) | Err(_) => Err(()),
            Ok(_) => Ok(&self.buffer),
        }
    }
}

impl Drop for IpcLinkMaster {
    fn drop(&mut self) {
        use wait_timeout::ChildExt;

        // the normal self.send function waits indefinitely for an answer that might not come if the subprocess
        // has already been stopped with IpcCommand::Exit
        _ = serde_json::to_writer(&mut self.tx, &IpcCommand::StopRecording);
        _ = self.tx.write(&[b'\n']);
        _ = serde_json::to_writer(&mut self.tx, &IpcCommand::Shutdown);
        _ = self.tx.write(&[b'\n']);
        _ = serde_json::to_writer(&mut self.tx, &IpcCommand::Exit);
        _ = self.tx.write(&[b'\n']);
        _ = self.tx.flush();

        match self.child_process.wait_timeout(Duration::from_secs(3)) {
            Ok(Some(status)) if status.success() => { /* process exited successfully */ }
            _ => _ = self.child_process.kill(),
        }
    }
}

pub struct IpcLinkSlave<'a> {
    tx: BufWriter<StdoutLock<'a>>,
    rx: BufReader<StdinLock<'a>>,
    buffer: String,
}

impl IpcLinkSlave<'_> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tx: BufWriter::new(io::stdout().lock()),
            rx: BufReader::new(io::stdin().lock()),
            buffer: String::with_capacity(512),
        }
    }

    pub fn respond(&mut self, mut handler: impl FnMut(IpcCommand) -> Option<IpcResponse>) {
        loop {
            let cmd = serde_json::from_str(self.read_line()).unwrap();

            let Some(response) = handler(cmd) else { break };
            _ = serde_json::to_writer::<_, IpcResponse>(&mut self.tx, &response);
            _ = self.tx.write(&[b'\n']);
            _ = self.tx.flush();
        }

        // Send one last IpcResponse::Ok because the other side is waiting for a response to IpcCommand::Exit
        _ = serde_json::to_writer::<_, IpcResponse>(&mut self.tx, &IpcResponse::Ok);
        _ = self.tx.write(&[b'\n']);
        _ = self.tx.flush();
    }

    fn read_line(&mut self) -> &str {
        self.buffer.clear();
        self.rx.read_line(&mut self.buffer).unwrap();
        &self.buffer
    }
}

impl Default for IpcLinkSlave<'_> {
    fn default() -> Self {
        Self::new()
    }
}
