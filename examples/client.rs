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

    writer.send(b"Hello from client").unwrap();

    let msg = reader.receive().unwrap();

    println!("Server: {}", String::from_utf8_lossy(&msg));
}