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

    pub fn create_pipe(path: &str) -> std::io::Result<()> {
        match mkfifo(path, Mode::S_IRWXU) {
            Ok(_) => {}
            Err(Errno::EEXIST) => {}
            Err(e) => return Err(std::io::Error::from_raw_os_error(e as i32)),
        }
        Ok(())
    }

    pub fn remove_pipe(path: &str) -> std::io::Result<()> {
        remove_file(path).or_else(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Ok(())
            } else {
                Err(e)
            }
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
            self.file.write_all(data)
        }

        pub fn receive(&mut self) -> std::io::Result<Vec<u8>> {
            let mut buf = vec![0u8; 4096];
            let size = self.file.read(&mut buf)?;
            buf.truncate(size);
            Ok(buf)
        }
    }
}

#[cfg(windows)]
mod platform {
    use super::*;
    use named_pipe::{PipeClient, PipeOptions};

    pub struct Pipe {
        pub client: PipeClient,
    }

    pub fn create_pipe(name: &str) -> std::io::Result<()> {
        PipeOptions::new(name).single()?;
        Ok(())
    }

    pub fn remove_pipe(_name: &str) -> std::io::Result<()> {
        Ok(())
    }

    pub fn open_read(name: &str) -> std::io::Result<Pipe> {
        let client = PipeClient::connect(name)?;
        Ok(Pipe { client })
    }

    pub fn open_write(name: &str) -> std::io::Result<Pipe> {
        let client = PipeClient::connect(name)?;
        Ok(Pipe { client })
    }

    impl Pipe {
        pub fn send(&mut self, data: &[u8]) -> std::io::Result<()> {
            self.client.write_all(data)
        }

        pub fn receive(&mut self) -> std::io::Result<Vec<u8>> {
            let mut buf = vec![0u8; 4096];
            let size = self.client.read(&mut buf)?;
            buf.truncate(size);
            Ok(buf)
        }
    }
}

pub use platform::*;