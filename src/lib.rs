use std::io::{Read, Write};

#[cfg(unix)]
mod platform {
    use super::*;
    use std::fs::{File, OpenOptions, remove_file};
    use nix::unistd::mkfifo;
    use nix::sys::stat::Mode;
    use nix::errno::Errno;

    pub struct Pipe {
        pub file: File,
    }

    pub fn create_server_read(path: &str) -> std::io::Result<Pipe> {
        match mkfifo(path, Mode::S_IRWXU) {
            Ok(_) | Err(Errno::EEXIST) => {}
            Err(e) => return Err(std::io::Error::from_raw_os_error(e as i32)),
        }
        let file = OpenOptions::new().read(true).open(path)?;
        Ok(Pipe { file })
    }

    pub fn create_server_write(path: &str) -> std::io::Result<Pipe> {
        match mkfifo(path, Mode::S_IRWXU) {
            Ok(_) | Err(Errno::EEXIST) => {}
            Err(e) => return Err(std::io::Error::from_raw_os_error(e as i32)),
        }
        let file = OpenOptions::new().write(true).open(path)?;
        Ok(Pipe { file })
    }

    pub fn remove_pipe(path: &str) -> std::io::Result<()> {
        remove_file(path).or_else(|e| {
            if e.kind() == std::io::ErrorKind::NotFound { Ok(()) } else { Err(e) }
        })
    }

    pub fn open_read(path: &str) -> std::io::Result<Pipe> {
        let file = OpenOptions::new().read(true).open(path)?;
        Ok(Pipe { file })
    }

    pub fn open_write(path: &str) -> std::io::Result<Pipe> {
        let file = OpenOptions::new().write(true).open(path)?;
        Ok(Pipe { file })
    }

    impl Pipe {
        pub fn send(&mut self, data: &[u8]) -> std::io::Result<()> {
            let len = (data.len() as u32).to_be_bytes();
            self.file.write_all(&len)?;
            self.file.write_all(data)
        }

        pub fn receive(&mut self) -> std::io::Result<Vec<u8>> {
            let mut len_buf = [0u8; 4];
            self.file.read_exact(&mut len_buf)?;
            let len = u32::from_be_bytes(len_buf) as usize;

            let mut buf = vec![0u8; len];
            self.file.read_exact(&mut buf)?;
            Ok(buf)
        }
    }
}

#[cfg(windows)]
mod platform {
    use super::*;
    use named_pipe::{PipeClient, PipeServer, PipeOptions};
    use std::thread;
    use std::time::Duration;

    pub enum Pipe {
        Client(PipeClient),
        Server(PipeServer),
    }

    pub fn remove_pipe(_name: &str) -> std::io::Result<()> {
        Ok(())
    }

    pub fn create_server_read(name: &str) -> std::io::Result<Pipe> {
        let server = PipeOptions::new(name).single()?.wait()?;
        Ok(Pipe::Server(server))
    }

    pub fn create_server_write(name: &str) -> std::io::Result<Pipe> {
        let server = PipeOptions::new(name).single()?.wait()?;
        Ok(Pipe::Server(server))
    }

    pub fn open_read(name: &str) -> std::io::Result<Pipe> {
        loop {
            match PipeClient::connect(name) {
                Ok(client) => return Ok(Pipe::Client(client)),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    thread::sleep(Duration::from_millis(50));
                }
                Err(e) => return Err(e),
            }
        }
    }

    pub fn open_write(name: &str) -> std::io::Result<Pipe> {
        loop {
            match PipeClient::connect(name) {
                Ok(client) => return Ok(Pipe::Client(client)),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    thread::sleep(Duration::from_millis(50));
                }
                Err(e) => return Err(e),
            }
        }
    }

    impl Pipe {
        pub fn send(&mut self, data: &[u8]) -> std::io::Result<()> {
            let len = (data.len() as u32).to_be_bytes();
            match self {
                Pipe::Client(c) => {
                    c.write_all(&len)?;
                    c.write_all(data)
                }
                Pipe::Server(s) => {
                    s.write_all(&len)?;
                    s.write_all(data)
                }
            }
        }

        pub fn receive(&mut self) -> std::io::Result<Vec<u8>> {
            let mut len_buf = [0u8; 4];
            match self {
                Pipe::Client(c) => c.read_exact(&mut len_buf)?,
                Pipe::Server(s) => s.read_exact(&mut len_buf)?,
            };
            
            let len = u32::from_be_bytes(len_buf) as usize;
            let mut buf = vec![0u8; len];
            
            match self {
                Pipe::Client(c) => c.read_exact(&mut buf)?,
                Pipe::Server(s) => s.read_exact(&mut buf)?,
            };
            
            Ok(buf)
        }
    }
}

pub use platform::*;