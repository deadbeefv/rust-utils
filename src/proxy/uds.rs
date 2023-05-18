use hyper::server::accept::Accept;
use rand::distributions::{Alphanumeric, DistString};
use std::io::Error;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::env::temp_dir;

#[cfg(not(target_os = "windows"))]
use tokio::net::{UnixListener, UnixStream};

#[cfg(target_os = "windows")]
use uds_windows::{UnixStream, UnixListener};

// #[cfg(target_os = "windows")]
// use tokio_uds_windows::UnixStream;
// #[cfg(target_os = "windows")]
// use tokio_uds_windows::UnixListener;

pub struct UDSConnector {
    inner: UnixListener,
    path: String,
}


impl UDSConnector {

    pub fn new(path: String) -> Result<Self, Error> {
        let uds = UnixListener::bind(&path)?;
        Ok(UDSConnector { inner: uds, path })
    }
    #[cfg(not(target_os = "windows"))]
    pub fn new_random() -> Result<Self, Error> {
        let mut rname = Alphanumeric.sample_string(&mut rand::thread_rng(), 8);
        rname = format!("/tmp/proxy-{}.sock", rname);
        Self::new(rname)
    }
    #[cfg(target_os = "windows")]
    pub fn new_random() -> Result<Self, Error> {
        let mut rname = Alphanumeric.sample_string(&mut rand::thread_rng(), 8);
        rname = format!("{}proxy-{}.sock", temp_dir().display(),rname);
        Self::new(rname)
    }
    pub fn get_path(&self) -> &str {
        &self.path
    }
}

impl Accept for UDSConnector {
    type Conn = UnixStream;
    type Error = Error;

    #[cfg(not(target_os = "windows"))]
    fn poll_accept(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        match self.inner.poll_accept {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok((socket, _addr))) => Poll::Ready(Some(Ok(socket))),
            Poll::Ready(Err(err)) => Poll::Ready(Some(Err(err))),
        }
    }
    #[cfg(target_os = "windows")]
    fn poll_accept(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        match self.inner.poll_accept() {
            Ok(socket) => Poll::Ready(Some(Ok(socket))),
            Err(e) => Poll::Ready(Some(Err(e)))
        }
    }

}

impl Drop for UDSConnector {
    fn drop(&mut self) {
        std::fs::remove_file(&self.path).unwrap();
    }
}
