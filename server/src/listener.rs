// Copyright (c) 2004-present, Facebook, Inc.
// All Rights Reserved.
//
// This software may be used and distributed according to the terms of the
// GNU General Public License version 2 or any later version.

use std::io;
use std::net::SocketAddr;

use failure::Error;
use futures::{Future, Stream};
use futures::sync::mpsc;
use futures_ext::{BoxFuture, BoxStream, FutureExt, StreamExt};

use bytes::Bytes;
use errors::*;
use tokio::net::{TcpListener, TcpStream};
use tokio_core::reactor::Remote;
use tokio_io::{AsyncRead, AsyncWrite, IoStream};
// TODO: (rain1) T30794235 move mononoke/server to tokio-codec
#[allow(deprecated)]
use tokio_io::codec::{FramedRead, FramedWrite};

use sshrelay::{Preamble, SshDecoder, SshEncoder, SshMsg, SshStream};

pub fn listener<P>(sockname: P) -> io::Result<IoStream<TcpStream>>
where
    P: AsRef<str>,
{
    let sockname = sockname.as_ref();
    let listener;
    let addr: SocketAddr = sockname.parse().unwrap();

    // First bind the socket. If the socket already exists then try connecting to it;
    // if there's no connection then replace it with a new one. (This assumes that simply
    // connecting is a no-op).
    loop {
        match TcpListener::bind(&addr) {
            Ok(l) => {
                listener = l;
                break;
            }
            Err(err) => {
                return Err(err);
            }
        }
    }

    Ok(listener.incoming().boxify())
}

pub struct Stdio {
    pub preamble: Preamble,
    pub stdin: BoxStream<Bytes, io::Error>,
    pub stdout: mpsc::Sender<Bytes>,
    pub stderr: mpsc::Sender<Bytes>,
}

// As a server, given a stream to a client, return an Io pair with stdin/stdout, and an
// auxillary sink for stderr.
pub fn ssh_server_mux<S>(s: S, remote: Remote) -> BoxFuture<Stdio, Error>
where
    S: AsyncRead + AsyncWrite + Send + 'static,
{
    let (rx, tx) = s.split();
    // TODO: (rain1) T30794235 move mononoke/server to tokio-codec
    #[allow(deprecated)]
    let wr = FramedWrite::new(tx, SshEncoder::new());
    #[allow(deprecated)]
    let rd = FramedRead::new(rx, SshDecoder::new());

    rd.into_future()
        .map_err(|_err| ErrorKind::ConnectionError.into())
        .and_then(move |(maybe_preamble, rd)| {
            let preamble = match maybe_preamble {
                Some(maybe_preamble) => {
                    if let SshStream::Preamble(preamble) = maybe_preamble.stream() {
                        preamble
                    } else {
                        return Err(ErrorKind::NoConnectionPreamble.into());
                    }
                }
                None => {
                    return Err(ErrorKind::NoConnectionPreamble.into());
                }
            };

            let stdin = rd.filter_map(|s| {
                if s.stream() == SshStream::Stdin {
                    Some(s.data())
                } else {
                    None
                }
            }).boxify();

            let (stdout, stderr) = {
                let (otx, orx) = mpsc::channel(1);
                let (etx, erx) = mpsc::channel(1);

                let orx = orx.map(|v| SshMsg::new(SshStream::Stdout, v));
                let erx = erx.map(|v| SshMsg::new(SshStream::Stderr, v));

                // Glue them together
                let fwd = orx.select(erx)
                    .map_err(|()| io::Error::new(io::ErrorKind::Other, "huh?"))
                    .forward(wr);

                // spawn a task for forwarding stdout/err into stream
                remote.spawn(|_handle| fwd.discard());

                (otx, etx)
            };

            Ok(Stdio {
                preamble,
                stdin,
                stdout,
                stderr,
            })
        })
        .boxify()
}
