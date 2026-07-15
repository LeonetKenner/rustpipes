use rustpipes::*;
use std::io::{self, Write};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let is_server = args.len() > 1 && args[1] == "server";

    // Cross-platform path routing
    #[cfg(unix)]
    let (pipe_in, pipe_out) = ("/tmp/rustpipe_a", "/tmp/rustpipe_b");
    
    #[cfg(windows)]
    let (pipe_in, pipe_out) = (r"\\.\pipe\rustpipe_a", r"\\.\pipe\rustpipe_b");

    if is_server {
        println!("🟢 Starting Server... Waiting for Client to connect.");
        
        let mut reader = PipeBuilder::new(pipe_in).server().read_only().build().unwrap();
        let mut writer = PipeBuilder::new(pipe_out).server().write_only().build().unwrap();
        
        println!("✅ Connected! Waiting for Client's first message...\n");

        loop {
            // 1. Wait for message from Client
            let msg = reader.receive().unwrap();
            println!("👤 Client: {}", String::from_utf8_lossy(&msg));
            
            // 2. Reply to Client
            print!("🗣️ You (Server): ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            writer.send(input.trim().as_bytes()).unwrap();
        }
    } else {
        println!("🔵 Starting Client... Connecting to Server.");
        
        let mut writer = PipeBuilder::new(pipe_in).client().write_only().build().unwrap();
        let mut reader = PipeBuilder::new(pipe_out).client().read_only().build().unwrap();
        
        println!("✅ Connected! Type your first message.\n");

        loop {
            // 1. Send message to Server
            print!("🗣️ You (Client): ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            writer.send(input.trim().as_bytes()).unwrap();

            // 2. Wait for reply from Server
            let msg = reader.receive().unwrap();
            println!("👤 Server: {}", String::from_utf8_lossy(&msg));
        }
    }
}