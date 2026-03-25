use rustpipes::*;

fn main() {
    #[cfg(unix)]
    let req = "/tmp/rustpipe_req";
    #[cfg(unix)]
    let res = "/tmp/rustpipe_res";

    #[cfg(windows)]
    let req = r"\\.\pipe\rustpipe_req";
    #[cfg(windows)]
    let res = r"\\.\pipe\rustpipe_res";

    let mut reader = create_server_read(req).unwrap();
    let mut writer = create_server_write(res).unwrap();

    let msg = reader.receive().unwrap();
    let text = String::from_utf8_lossy(&msg);

    println!("Server received exactly {} bytes.", msg.len());
    println!("Message Content:\n{}\n", text);

    let large_response = "SERVER_DATA_CHUNK_".repeat(300).into_bytes();
    writer.send(&large_response).unwrap();
    
    remove_pipe(req).unwrap();
    remove_pipe(res).unwrap();
}