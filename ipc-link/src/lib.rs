use std::{
    ffi::OsStr,
    io::{self, BufRead, BufReader, BufWriter, StdinLock, StdoutLock, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

#[derive(serde::Serialize, serde::Deserialize)]
pub enum IpcCommand {
    Init {
        libobs_data_path: Option<String>,
        plugin_bin_path: Option<String>,
        plugin_data_path: Option<String>,
    },
    Settings(intprocess_recorder::RecorderSettings),
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

pub struct IpcLinkMaster {
    tx: BufWriter<ChildStdin>,
    rx: BufReader<ChildStdout>,
    buffer: String,
    child_process: Child,
}

impl IpcLinkMaster {
    pub fn new(executable: impl AsRef<OsStr>) -> Self {
        let mut child_process = Command::new(executable)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        Self {
            tx: BufWriter::new(child_process.stdin.take().unwrap()),
            rx: BufReader::new(child_process.stdout.take().unwrap()),
            buffer: String::new(),
            child_process,
        }
    }

    pub fn send(&mut self, cmd: IpcCommand) -> bool {
        _ = serde_json::to_writer(&mut self.tx, &cmd);
        _ = self.tx.write(&['\n' as u8]);
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
        // the normal self.send function waits indefinitely for an answer that might not come if the subprocess
        // has already been stopped with IpcCommand::Exit
        _ = serde_json::to_writer(&mut self.tx, &IpcCommand::StopRecording);
        _ = self.tx.write(&['\n' as u8]);
        _ = serde_json::to_writer(&mut self.tx, &IpcCommand::Shutdown);
        _ = self.tx.write(&['\n' as u8]);
        _ = serde_json::to_writer(&mut self.tx, &IpcCommand::Exit);
        _ = self.tx.write(&['\n' as u8]);
        _ = self.tx.flush();

        // # Warning (from the std lib docs)
        //
        // On some systems, calling [`wait`] or similar is necessary for the OS to
        // release resources. A process that terminated but has not been waited on is
        // still around as a "zombie". Leaving too many zombies around may exhaust
        // global resources (for example process IDs).
        //
        // The standard library does *not* automatically wait on child processes (not
        // even if the `Child` is dropped), it is up to the application developer to do
        // so. As a consequence, dropping `Child` handles without waiting on them first
        // is not recommended in long-running applications.
        _ = self.child_process.wait();
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
            buffer: String::new(),
        }
    }

    pub fn respond<H: FnMut(IpcCommand) -> Option<IpcResponse>>(&mut self, mut handler: H) {
        loop {
            let cmd = serde_json::from_str(self.read_line()).unwrap();

            let Some(response) = handler(cmd) else { break };
            _ = serde_json::to_writer::<_, IpcResponse>(&mut self.tx, &response);
            _ = self.tx.write(&['\n' as u8]);
            _ = self.tx.flush();
        }

        // Send one last IpcResponse::Ok because the other side is waiting for a response to IpcCommand::Exit
        _ = serde_json::to_writer::<_, IpcResponse>(&mut self.tx, &IpcResponse::Ok);
        _ = self.tx.write(&['\n' as u8]);
        _ = self.tx.flush();
    }

    fn read_line(&mut self) -> &str {
        self.buffer.clear();
        self.rx.read_line(&mut self.buffer).unwrap();
        &self.buffer
    }
}
