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

    let mut writer = open_write(req).unwrap();
    let mut reader = open_read(res).unwrap();

    let large_payload = "CLIENT_DATA_CHUNK_".repeat(300).into_bytes();
    writer.send(&large_payload).unwrap();

    let msg = reader.receive().unwrap();
    let text = String::from_utf8_lossy(&msg);

    println!("Client received exactly {} bytes.", msg.len());
    println!("Message Content:\n{}\n", text);
}