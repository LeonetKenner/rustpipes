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

    let mut reader = PipeBuilder::new(req)
        .server()
        .read_only()
        .build()
        .unwrap();

    let mut writer = PipeBuilder::new(res)
        .server()
        .write_only()
        .build()
        .unwrap();

    let msg = reader.receive().unwrap();
    let text = String::from_utf8_lossy(&msg);

    println!("Server received exactly {} bytes.", msg.len());
    println!("Message Content:\n{}\n", text);

    let large_response = "SERVER_DATA_CHUNK_".repeat(300).into_bytes();
    writer.send(&large_response).unwrap();
    
    Pipe::remove(req).unwrap();
    Pipe::remove(res).unwrap();
}