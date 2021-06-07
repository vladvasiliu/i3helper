use log::{error, info, warn};
use std::error::Error as StdError;
use std::fmt;
use tokio::signal::ctrl_c;
use tokio::signal::unix::{signal, SignalKind};
use tokio_i3ipc::event::{Event as I3Event, Subscribe, WindowChange};
use tokio_i3ipc::I3;

struct WindowStack(Vec<usize>);

/// A stack of Windows, ordered by when they were last focused. The last one is currently focused.
impl WindowStack {
    pub fn new() -> Self {
        // 128 is fairly random, but seems like a reasonable number of open windows.
        Self::with_capacity(128)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    pub fn push(&mut self, win_id: usize) {
        self.delete(win_id);
        self.0.push(win_id);
    }

    pub fn delete(&mut self, win_id: usize) {
        self.0.retain(|x| *x != win_id);
    }

    pub fn set_active(&mut self, win_id: usize) {
        self.push(win_id)
    }

    pub fn get_active(&self) -> Option<&usize> {
        self.0.last()
    }

    pub fn get_previous(&self) -> Option<&usize> {
        let len = self.0.len();
        if len > 1 {
            self.0.get(len - 2)
        } else {
            None
        }
    }
}

pub struct I3Manager {
    i3: I3,
    window_stack: WindowStack,
}

impl I3Manager {
    pub async fn new() -> Result<I3Manager> {
        let i3 = I3::connect().await?;
        Ok(Self {
            i3,
            window_stack: WindowStack::new(),
        })
    }

    /** The main event loop

    Implementation notes:

    `i3-ipc` doesn't handle concurrency well as the message exchanges seem to be sequential.
    There's a pathologic when events are sent in between commands and their answers, eg:
    1. send focus command
    2. i3 updates focus and sends a `Window` type event
    3. i3 sends the response to the focus command

    The way `i3-ipc` implements `run_command` is with a `send_msg` immediately followed by
    `read_msg`. This breaks if an event is sent in the meantime, which can happen even as a result
    of the sent message (see above).

    To work around this, the following is done:
    1. Have a dedicated connection with i3 for events. No messaging happens over this.
    2. Have a dedicated connection with i3 for commands, not subscribed to anything, so no events.
    3. Have an event loop for sending commands to i3. This ensures that responses are consumed
        before sending new messages.
     */
    pub async fn run(&mut self) -> Result<()> {
        let mut i3_event_connection = I3::connect().await?;
        i3_event_connection.subscribe([Subscribe::Window]).await?;

        let mut usr1_stream = signal(SignalKind::user_defined1())?;

        loop {
            tokio::select! {
                i3_event = i3_event_connection.read_event() => {
                    match i3_event {
                        Ok(i3_event) => self.handle_i3_event(i3_event).await?,
                        Err(err) => error!("Got i3 error: {:#?}", err)
                    }
                }
                _ = ctrl_c() => {
                    info!("Received ^C, shutting down");
                    break;
                }
                _ = usr1_stream.recv() => {
                    self.switch_to_previous().await?;
                }
            }
        }

        Ok(())
    }

    async fn handle_i3_event(&mut self, event: I3Event) -> Result<()> {
        // Can't pattern match on a box in stable rust (june 2021)
        // https://doc.rust-lang.org/stable/unstable-book/language-features/box-patterns.html
        if let I3Event::Window(event) = event {
            match event.change {
                WindowChange::Focus => self.window_stack.set_active(event.container.id),
                WindowChange::Close => self.window_stack.delete(event.container.id),
                _ => (),
            }
        }
        Ok(())
    }

    async fn switch_to_previous(&mut self) -> Result<()> {
        if let Some(prev_window_id) = self.window_stack.get_previous() {
            let payload = format!("[con_id={}] focus", prev_window_id);
            self.run_command(payload).await?;
        }
        Ok(())
    }

    async fn run_command<P: AsRef<str> + fmt::Display>(&mut self, payload: P) -> Result<()> {
        let resp = self.i3.run_command(&payload).await?;
        let resp: Vec<&String> = resp.iter().filter_map(|x| x.error.as_ref()).collect();
        if !resp.is_empty() {
            warn!(
                "I3 Command failed. Command: {} - Response: {:?}",
                &payload, resp
            );
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {}

impl StdError for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!()
    }
}

impl From<tokio::io::Error> for Error {
    fn from(_: tokio::io::Error) -> Self {
        unimplemented!()
    }
}

pub type Result<T> = std::result::Result<T, Error>;
