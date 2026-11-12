//! One running copy per data folder.
//!
//! The app keeps running after its window closes, so launching it again must show
//! the window of the copy that is already running instead of starting a second
//! monitor on the same history file.
//!
//! The running copy listens on a loopback port and records the port in
//! `instance.port`. A new copy asks that port to open the window and exits when it
//! gets the expected answer.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::path::Path;
use std::time::Duration;

use crate::command::{Command, CommandSender};

const PORT_FILE: &str = "instance.port";
const OPEN: &str = "internet-speed-monitor open";
const PING: &str = "internet-speed-monitor ping";
const ACK: &str = "internet-speed-monitor ok";
const TIMEOUT: Duration = Duration::from_millis(500);

pub enum Instance {
    /// No other copy answered: this one runs. `None` when listening failed, in which
    /// case later launches cannot find this copy.
    Primary(Option<Listener>),
    /// Another copy is running and has been told what to do.
    AlreadyRunning,
}

pub struct Listener {
    socket: TcpListener,
}

/// Look for a running copy. `open_window` asks it to show its window; a background
/// launch only checks that it is there.
pub fn acquire(data_dir: &Path, open_window: bool) -> Instance {
    let port_file = data_dir.join(PORT_FILE);
    let request = if open_window { OPEN } else { PING };
    if let Some(port) = read_port(&port_file)
        && ask(port, request)
    {
        return Instance::AlreadyRunning;
    }
    match listen(&port_file) {
        Ok(listener) => Instance::Primary(Some(listener)),
        Err(error) => {
            log::error!("Single-instance listener failed: {error}");
            Instance::Primary(None)
        }
    }
}

fn read_port(port_file: &Path) -> Option<u16> {
    std::fs::read_to_string(port_file).ok()?.trim().parse().ok()
}

fn ask(port: u16, request: &str) -> bool {
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    let Ok(mut stream) = TcpStream::connect_timeout(&address, TIMEOUT) else {
        return false;
    };
    let _ = stream.set_read_timeout(Some(TIMEOUT));
    let _ = stream.set_write_timeout(Some(TIMEOUT));
    if writeln!(stream, "{request}").is_err() {
        return false;
    }
    // Another program may have been given the port of a copy that crashed, so only
    // the exact answer counts.
    read_line(&stream).is_some_and(|line| line == ACK)
}

fn listen(port_file: &Path) -> std::io::Result<Listener> {
    let socket = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
    std::fs::write(port_file, socket.local_addr()?.port().to_string())?;
    Ok(Listener { socket })
}

fn read_line(stream: &TcpStream) -> Option<String> {
    let mut line = String::new();
    BufReader::new(stream.take(64)).read_line(&mut line).ok()?;
    Some(line.trim_end().to_string())
}

impl Listener {
    /// Answer later launches for as long as the process lives.
    pub fn serve(self, commands: CommandSender) {
        let spawned = std::thread::Builder::new()
            .name("ism-instance".to_string())
            .spawn(move || {
                for stream in self.socket.incoming().flatten() {
                    let _ = stream.set_read_timeout(Some(TIMEOUT));
                    let _ = stream.set_write_timeout(Some(TIMEOUT));
                    let request = read_line(&stream);
                    let open = request.as_deref() == Some(OPEN);
                    if open || request.as_deref() == Some(PING) {
                        let _ = writeln!(&stream, "{ACK}");
                    }
                    if open {
                        log::info!("Launched again: showing the window");
                        commands.send(Command::Open);
                    }
                }
            });
        if let Err(error) = spawned {
            log::error!("Single-instance listener thread failed: {error}");
        }
    }
}

/// Forget the port on exit so the next launch does not try a dead one.
pub fn release(data_dir: &Path) {
    let _ = std::fs::remove_file(data_dir.join(PORT_FILE));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wake::Waker;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("ism-instance-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn second_launch_opens_the_window_of_the_first() {
        let dir = temp_dir("open");
        let Instance::Primary(Some(listener)) = acquire(&dir, true) else {
            panic!("first launch must become the running copy");
        };
        let (sender, commands) = crate::command::channel(Waker::window_only());
        listener.serve(sender);

        assert!(matches!(acquire(&dir, true), Instance::AlreadyRunning));
        assert_eq!(
            commands.recv_timeout(Duration::from_secs(5)),
            Ok(Command::Open)
        );

        // A background launch finds the running copy and leaves its window alone.
        assert!(matches!(acquire(&dir, false), Instance::AlreadyRunning));
        assert!(commands.recv_timeout(Duration::from_millis(300)).is_err());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn stale_port_file_does_not_block_a_launch() {
        let dir = temp_dir("stale");
        // A port nobody answers on with the expected reply.
        let unused = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = unused.local_addr().unwrap().port();
        drop(unused);
        std::fs::write(dir.join(PORT_FILE), port.to_string()).unwrap();

        assert!(matches!(acquire(&dir, true), Instance::Primary(Some(_))));
        let _ = std::fs::remove_dir_all(dir);
    }
}
