use ipc_link::{IpcCommand, IpcLinkMaster};

fn main() {
    let mut link = IpcLinkMaster::new("extprocess_recorder.exe");
    link.send(IpcCommand::Init {
        libobs_data_path: None,
        plugin_bin_path: None,
        plugin_data_path: None,
    });

    println!("Stop: {}", link.send(IpcCommand::StopRecording));
    println!("Start: {}", link.send(IpcCommand::StartRecording));
    std::thread::sleep(std::time::Duration::from_secs(3));
    println!("Stop: {}", link.send(IpcCommand::StopRecording));
    println!("Exit: {}", link.send(IpcCommand::Exit));
    println!("Shutdown: {}", link.send(IpcCommand::Shutdown));
    println!("Exit: {}", link.send(IpcCommand::Exit));
}
