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

    create_pipe(req).unwrap();
    create_pipe(res).unwrap();

    let mut reader = open_read(req).unwrap();
    let mut writer = open_write(res).unwrap();

    let msg = reader.receive().unwrap();

    println!("Client: {}", String::from_utf8_lossy(&msg));

    writer.send(b"Hello from server").unwrap();
}