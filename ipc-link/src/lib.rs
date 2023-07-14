use std::{
    ffi::OsStr,
    io::{self, BufRead, BufReader, BufWriter, StdinLock, StdoutLock, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    time::Duration,
};

#[derive(serde::Serialize, serde::Deserialize)]
pub enum IpcCommand {
    Init {
        libobs_data_path: Option<String>,
        plugin_bin_path: Option<String>,
        plugin_data_path: Option<String>,
    },
    Settings(intprocess_recorder::settings::RecorderSettings),
    StartRecording,
    StopRecording,
    Shutdown,
    Exit,
}

#[derive(PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum IpcResponse {
    Ok,
    Err(String),
    Log(String),
}

#[derive(Debug)]
pub struct IpcLinkMaster {
    tx: BufWriter<ChildStdin>,
    rx: BufReader<ChildStdout>,
    buffer: String,
    child_process: Child,
}

impl IpcLinkMaster {
    pub fn new(executable: impl AsRef<OsStr>) -> io::Result<Self> {
        let mut child_process = Command::new(executable)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        Ok(Self {
            tx: BufWriter::new(child_process.stdin.take().unwrap()),
            rx: BufReader::new(child_process.stdout.take().unwrap()),
            buffer: String::with_capacity(512),
            child_process,
        })
    }

    pub fn send(&mut self, cmd: IpcCommand) -> bool {
        _ = serde_json::to_writer(&mut self.tx, &cmd);
        _ = self.tx.write(&[b'\n']);
        _ = self.tx.flush();

        loop {
            match serde_json::from_str::<IpcResponse>(self.read_line()).unwrap() {
                IpcResponse::Ok => return true,
                IpcResponse::Err(e) => {
                    println!("ipc_link error: {e}");
                    return false;
                }
                IpcResponse::Log(log) => println!("ipc_link log: {log}"),
            }
        }
    }

    fn read_line(&mut self) -> &str {
        self.buffer.clear();
        self.rx.read_line(&mut self.buffer).unwrap();
        &self.buffer
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

        match self.child_process.wait_timeout(Duration::from_secs(1)) {
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
    pub fn new() -> Self {
        Self {
            tx: BufWriter::new(io::stdout().lock()),
            rx: BufReader::new(io::stdin().lock()),
            buffer: String::with_capacity(512),
        }
    }

    pub fn respond<H: FnMut(IpcCommand) -> Option<IpcResponse>>(&mut self, mut handler: H) {
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
