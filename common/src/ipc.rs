use std::{
    env, fs,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    os::{
        fd::AsFd,
        unix::net::{UnixListener, UnixStream},
    },
    path::Path,
};

use bincode::{config, decode_from_std_read, encode_into_std_write};

use crate::types::{IntoRequest, Request, Response};

#[derive(Debug)]
pub enum IpcError {
    Io(std::io::Error),
    Encoding(bincode::error::EncodeError),
    Decoding(bincode::error::DecodeError),
    InvalidResponse,
    ConnectionClosed,
}

pub struct IpcSocket<T> {
    data: T,
    marker: PhantomData<T>,
}

pub struct Listener(UnixListener);
pub struct Stream(UnixStream);

// Configuration for bincode serialization
fn bincode_config() -> config::Configuration {
    config::standard()
}

impl AsFd for Listener {
    fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl<T> IpcSocket<T> {
    fn new(data: T) -> Self {
        Self {
            data,
            marker: Default::default(),
        }
    }

    pub fn getuid() -> u32 {
        use std::os::unix::fs::MetadataExt;
        std::fs::metadata("/proc/self").map(|m| m.uid()).unwrap()
    }

    pub fn socket_file() -> String {
        let runtime =
            env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| format!("/run/user/{}", Self::getuid()));

        let display = if let Ok(wayland_socket) = std::env::var("WAYLAND_DISPLAY") {
            let mut i = 0;
            // if WAYLAND_DISPLAY is a full path, use only its final component
            for (j, ch) in wayland_socket.bytes().enumerate().rev() {
                if ch == b'/' {
                    i = j + 1;
                    break;
                }
            }
            (wayland_socket[i..]).to_string()
        } else {
            eprintln!("WARNING: WAYLAND_DISPLAY variable not set. Defaulting to wayland-0");
            "wayland-0.sock".to_string()
        };

        format!("{runtime}/wlrs-{display}.sock")
    }
}

impl<T> Deref for IpcSocket<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for IpcSocket<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl IpcSocket<Listener> {
    pub fn listen() -> Result<Self, IpcError> {
        let socket_file = Self::socket_file();

        // Make sure the parent directory exists
        if let Some(parent) = Path::new(&socket_file).parent() {
            fs::create_dir_all(parent).map_err(IpcError::Io)?;
        }

        // Remove the socket file if it already exists
        if Path::new(&socket_file).exists() {
            fs::remove_file(&socket_file).map_err(IpcError::Io)?;
        }

        let listener = UnixListener::bind(&socket_file).map_err(IpcError::Io)?;
        Ok(Self::new(Listener(listener)))
    }

    pub fn accept(&self) -> Result<IpcSocket<Stream>, IpcError> {
        let (stream, _) = self.0.accept().map_err(IpcError::Io)?;
        Ok(IpcSocket::new(Stream(stream)))
    }

    pub fn handle_request<F>(&self, handler: F) -> Result<(), IpcError>
    where
        F: Fn(Request) -> Response,
    {
        let (stream, _) = self.0.accept().map_err(IpcError::Io)?;
        let mut client = IpcSocket::new(Stream(stream));

        // Receive request
        let request: Request = client.receive()?;

        // Process request
        let response = handler(request);

        // Send response
        client.send(&response)?;

        Ok(())
    }
}

impl IpcSocket<Stream> {
    pub fn connect() -> Result<Self, IpcError> {
        let socket_file = Self::socket_file();
        let stream = UnixStream::connect(&socket_file).map_err(IpcError::Io)?;
        Ok(Self::new(Stream(stream)))
    }

    pub fn send<T: bincode::Encode>(&mut self, message: &T) -> Result<usize, IpcError> {
        encode_into_std_write(message, &mut self.0, bincode_config()).map_err(IpcError::Encoding)
    }

    pub fn receive<T: bincode::Decode<()>>(&mut self) -> Result<T, IpcError> {
        decode_from_std_read(&mut self.0, bincode_config()).map_err(IpcError::Decoding)
    }

    pub fn request<R: IntoRequest>(&mut self, request: R) -> Result<R::Response, IpcError>
    where
        R::Response: TryFrom<Response, Error = ()>,
    {
        // Send request
        self.send(&request.into_request())?;

        // Receive response
        let response: Response = self.receive()?;

        // Convert to expected response type
        response.try_into().map_err(|_| IpcError::InvalidResponse)
    }

    pub fn is_daemon_running() -> bool {
        let socket_file = Self::socket_file();
        UnixStream::connect(&socket_file).is_ok()
    }
}
