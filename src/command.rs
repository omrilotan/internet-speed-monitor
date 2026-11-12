//! Requests from outside the window: the tray icon, a second launch of the app, the
//! operating system. Whoever owns the event loop at the time acts on them.

use std::sync::mpsc::{Receiver, Sender};

use crate::wake::Waker;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    /// Show the window, or bring it to the front.
    Open,
    /// Stop monitoring and exit.
    Quit,
}

#[derive(Clone)]
pub struct CommandSender {
    tx: Sender<Command>,
    waker: Waker,
}

impl CommandSender {
    pub fn send(&self, command: Command) {
        let _ = self.tx.send(command);
        self.waker.wake();
    }
}

pub fn channel(waker: Waker) -> (CommandSender, Receiver<Command>) {
    let (tx, rx) = std::sync::mpsc::channel();
    (CommandSender { tx, waker }, rx)
}
