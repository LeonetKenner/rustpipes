use std::io::{self, Read, Write};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PipeRole {
    Server,
    Client,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PipeDirection {
    Read,
    Write,
    Duplex,
}

pub struct PipeBuilder {
    path: String,
    role: PipeRole,
    direction: PipeDirection,
}

impl PipeBuilder {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            role: PipeRole::Client,
            direction: PipeDirection::Duplex,
        }
    }

    pub fn server(mut self) -> Self {
        self.role = PipeRole::Server;
        self
    }

    pub fn client(mut self) -> Self {
        self.role = PipeRole::Client;
        self
    }

    pub fn read_only(mut self) -> Self {
        self.direction = PipeDirection::Read;
        self
    }

    pub fn write_only(mut self) -> Self {
        self.direction = PipeDirection::Write;
        self
    }

    pub fn duplex(mut self) -> Self {
        self.direction = PipeDirection::Duplex;
        self
    }

    pub fn build(self) -> io::Result<Pipe> {
        let inner = match self.role {
            PipeRole::Server => platform::create_server(&self.path, self.direction)?,
            PipeRole::Client => platform::open_client(&self.path, self.direction)?,
        };
        Ok(Pipe { inner })
    }
}

pub struct Pipe {
    inner: platform::InnerPipe,
}

impl Pipe {
    pub fn send(&mut self, data: &[u8]) -> io::Result<()> {
        let len = (data.len() as u32).to_be_bytes();
        self.write_all(&len)?;
        self.write_all(data)?;
        self.flush()
    }

    pub fn receive(&mut self) -> io::Result<Vec<u8>> {
        let mut len_buf = [0u8; 4];
        self.read_exact(&mut len_buf)?;
        let len = u32::from_be_bytes(len_buf) as usize;

        let mut buf = vec![0u8; len];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    pub fn remove(path: &str) -> io::Result<()> {
        platform::remove(path)
    }
}

impl Read for Pipe {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl Write for Pipe {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

#[cfg(unix)]
mod platform {
    use super::PipeDirection;
    use nix::errno::Errno;
    use nix::sys::stat::Mode;
    use nix::unistd::mkfifo;
    use std::fs::{remove_file, File, OpenOptions};
    use std::io::{self, Read, Write};
    use std::thread;
    use std::time::Duration;

    pub struct InnerPipe(File);

    pub fn create_server(path: &str, direction: PipeDirection) -> io::Result<InnerPipe> {
        match mkfifo(path, Mode::S_IRWXU) {
            Ok(_) | Err(Errno::EEXIST) => {}
            Err(e) => return Err(io::Error::from_raw_os_error(e as i32)),
        }
        open_client(path, direction)
    }

    pub fn open_client(path: &str, direction: PipeDirection) -> io::Result<InnerPipe> {
        let mut opts = OpenOptions::new();
        match direction {
            PipeDirection::Read => opts.read(true),
            PipeDirection::Write => opts.write(true),
            PipeDirection::Duplex => opts.read(true).write(true),
        };

        let backoff = Duration::from_millis(50);
        let mut retries = 0;
        
        // Retry loop: Gives the server up to ~5 seconds to create the FIFO
        loop {
            match opts.open(path) {
                Ok(file) => return Ok(InnerPipe(file)),
                Err(e) if e.kind() == io::ErrorKind::NotFound && retries < 100 => {
                    thread::sleep(backoff);
                    retries += 1;
                }
                Err(e) => return Err(e),
            }
        }
    }

    pub fn remove(path: &str) -> io::Result<()> {
        match remove_file(path) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            res => res,
        }
    }

    impl Read for InnerPipe {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.0.read(buf)
        }
    }

    impl Write for InnerPipe {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.0.flush()
        }
    }
}

#[cfg(windows)]
mod platform {
    use super::PipeDirection;
    use named_pipe::{PipeClient, PipeOptions as WinPipeOptions, PipeServer};
    use std::io::{self, Read, Write};
    use std::thread;
    use std::time::Duration;

    pub enum InnerPipe {
        Client(PipeClient),
        Server(PipeServer),
    }

    pub fn create_server(name: &str, _direction: PipeDirection) -> io::Result<InnerPipe> {
        let server = WinPipeOptions::new(name).single()?.wait()?;
        Ok(InnerPipe::Server(server))
    }

    pub fn open_client(name: &str, _direction: PipeDirection) -> io::Result<InnerPipe> {
        let backoff = Duration::from_millis(50);
        let mut retries = 0;
        
        loop {
            match PipeClient::connect(name) {
                Ok(client) => return Ok(InnerPipe::Client(client)),
                Err(e) if e.kind() == io::ErrorKind::NotFound && retries < 100 => {
                    thread::sleep(backoff);
                    retries += 1;
                }
                Err(e) => return Err(e),
            }
        }
    }

    pub fn remove(_name: &str) -> io::Result<()> {
        Ok(())
    }

    impl Read for InnerPipe {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            match self {
                InnerPipe::Client(c) => c.read(buf),
                InnerPipe::Server(s) => s.read(buf),
            }
        }
    }

    impl Write for InnerPipe {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            match self {
                InnerPipe::Client(c) => c.write(buf),
                InnerPipe::Server(s) => s.write(buf),
            }
        }

        fn flush(&mut self) -> io::Result<()> {
            match self {
                InnerPipe::Client(c) => c.flush(),
                InnerPipe::Server(s) => s.flush(),
            }
        }
    }
}
#[cfg(target_arch = "wasm32")]
mod platform {
    use super::PipeDirection;
    use std::collections::VecDeque;
    use std::io::{self, Read, Write, Error, ErrorKind};
    use std::sync::{Arc, Mutex};
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;
    use web_sys::{BroadcastChannel, MessageEvent};
    use js_sys::Uint8Array;

    pub struct InnerPipe {
        channel: BroadcastChannel,
        buffer: Arc<Mutex<VecDeque<u8>>>,
        _listener: Closure<dyn FnMut(MessageEvent)>,
    }
    pub fn create_server(name: &str, _direction: PipeDirection) -> io::Result<InnerPipe> {
        setup_broadcast_channel(name)
    }

    pub fn open_client(name: &str, _direction: PipeDirection) -> io::Result<InnerPipe> {
        setup_broadcast_channel(name)
    }

    fn setup_broadcast_channel(name: &str) -> io::Result<InnerPipe> {
        let channel = BroadcastChannel::new(name).map_err(|_| {
            Error::new(ErrorKind::Other, "Failed to create BroadcastChannel")
        })?;

        let buffer = Arc::new(Mutex::new(VecDeque::new()));
        let buffer_clone = buffer.clone();
        let listener = Closure::wrap(Box::new(move |event: MessageEvent| {
            if let Ok(js_array) = event.data().dyn_into::<Uint8Array>() {
                let mut data = vec![0; js_array.length() as usize];
                js_array.copy_to(&mut data);
                if let Ok(mut buf) = buffer_clone.lock() {
                    buf.extend(data);
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        channel.set_onmessage(Some(listener.as_ref().unchecked_ref()));

        Ok(InnerPipe {
            channel,
            buffer,
            _listener: listener,
        })
    }

    pub fn remove(name: &str) -> io::Result<()> {
        Ok(())
    }

    impl Drop for InnerPipe {
        fn drop(&mut self) {
            self.channel.close();
            self.channel.set_onmessage(None);
        }
    }

    impl Read for InnerPipe {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let mut inner = self.buffer.lock().unwrap();
            
            if inner.is_empty() {
                return Err(Error::new(ErrorKind::WouldBlock, "WASM pipe has no new data"));
            }

            let mut bytes_read = 0;
            for b in buf.iter_mut() {
                if let Some(val) = inner.pop_front() {
                    *b = val;
                    bytes_read += 1;
                } else {
                    break;
                }
            }
            Ok(bytes_read)
        }
    }

    impl Write for InnerPipe {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let js_array = Uint8Array::from(buf);
            self.channel.post_message(&js_array).map_err(|_| {
                Error::new(ErrorKind::Other, "Failed to post message to BroadcastChannel")
            })?;
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
}