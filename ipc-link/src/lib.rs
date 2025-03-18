use std::{
    io::{self, BufRead, BufReader, BufWriter, StdinLock, StdoutLock, Write},
    path::{Component, Path},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    time::Duration,
};

use intprocess_recorder::settings::{Adapter, Encoder, RecorderSettings};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum IpcCommand {
    Init {
        libobs_data_path: Option<String>,
        plugin_bin_path: Option<String>,
        plugin_data_path: Option<String>,
    },
    Configure(RecorderSettings),
    Encoders,
    Adapter,
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
    Adapter(Adapter),
    Recording(bool),
    Err(String),
}

#[derive(Debug)]
pub struct IpcLinkMaster {
    tx: BufWriter<ChildStdin>,
    rx: BufReader<ChildStdout>,
    buffer: String,
    child_process: Child,
}

impl IpcLinkMaster {
    pub fn new(executable: impl AsRef<Path>) -> io::Result<Self> {
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
        })
    }

    pub fn send(&mut self, cmd: IpcCommand) -> IpcResponse {
        if let Err(e) = serde_json::to_writer(&mut self.tx, &cmd) {
            return IpcResponse::Err(format!("{e:?}"));
        }
        if let Err(e) = self.tx.write(b"\n") {
            return IpcResponse::Err(format!("{e:?}"));
        }
        if let Err(e) = self.tx.flush() {
            return IpcResponse::Err(format!("{e:?}"));
        }

        loop {
            let line = match self.read_line() {
                Ok(line) => line,
                Err(e) => return IpcResponse::Err(format!("failed to read from recorder: {e}")),
            };
            match serde_json::from_str::<IpcResponse>(line) {
                Ok(response) => return response,
                // trim newlines from the end because log::info!() adds one
                Err(_) => log::info!("[rec]: {}", line.trim_end()),
            }
        }
    }

    pub fn drain_logs(&mut self) {
        while let Ok(line) = self.read_line() {
            // trim newlines from the end because log::info!() adds one
            log::info!("[rec]: {}", line.trim_end());
        }
    }

    fn read_line(&mut self) -> Result<&str, String> {
        self.buffer.clear();

        match self.rx.read_line(&mut self.buffer) {
            Ok(0) => Err("EOF".into()),
            Err(e) => Err(format!("{e:?}")),
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
        _ = self.tx.write(b"\n");
        _ = serde_json::to_writer(&mut self.tx, &IpcCommand::Shutdown);
        _ = self.tx.write(b"\n");
        _ = serde_json::to_writer(&mut self.tx, &IpcCommand::Exit);
        _ = self.tx.write(b"\n");
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
            _ = self.tx.write(b"\n");
            _ = self.tx.flush();
        }

        // Send one last IpcResponse::Ok because the other side is waiting for a response to IpcCommand::Exit
        _ = serde_json::to_writer::<_, IpcResponse>(&mut self.tx, &IpcResponse::Ok);
        _ = self.tx.write(b"\n");
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
