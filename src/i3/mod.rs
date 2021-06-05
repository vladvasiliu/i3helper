use futures::stream::StreamExt;
use log::warn;
use std::error::Error as StdError;
use std::fmt;
use tokio::sync::mpsc::{channel, error::SendError, Receiver, Sender};
use tokio_i3ipc::event::{Event as I3Event, Subscribe, WindowChange};
use tokio_i3ipc::reply::Node;
use tokio_i3ipc::I3;

pub async fn focus_listener(event_tx: Sender<EventType>) -> Result<()> {
    let mut i3 = I3::connect().await?;
    i3.subscribe([Subscribe::Window]).await?;

    let mut listener = i3.listen();
    while let Some(event) = listener.next().await {
        match event? {
            I3Event::Window(ev) if ev.change == WindowChange::Focus => {
                event_tx.send(EventType::FocusChange(ev.container)).await?;
            }
            _ => (),
        }
    }
    Ok(())
}

#[derive(Debug)]
pub enum EventType {
    FocusChange(Node),
}

pub struct I3Manager {
    prev_window_id: Option<usize>,
    curr_window_id: Option<usize>,
    i3_event_rx: Receiver<EventType>,
    i3_event_tx: Sender<EventType>,
    i3: I3,
}

impl I3Manager {
    pub async fn new() -> Result<I3Manager> {
        let (event_tx, event_rx) = channel(10);
        let i3 = I3::connect().await?;
        Ok(Self {
            prev_window_id: None,
            curr_window_id: None,
            i3_event_rx: event_rx,
            i3_event_tx: event_tx,
            i3,
        })
    }

    pub fn get_i3_event_tx(&self) -> Sender<EventType> {
        self.i3_event_tx.clone()
    }

    pub async fn i3_cmd_sender(&mut self) -> Result<()> {
        while let Some(event) = self.i3_event_rx.recv().await {
            self.handle_event(event).await?;
        }
        Ok(())
    }

    async fn handle_event(&mut self, event: EventType) -> Result<()> {
        match event {
            EventType::FocusChange(node) => self.focus_change(node).await?,
        }
        Ok(())
    }

    async fn focus_change(&mut self, new_node: Node) -> Result<()> {
        if let Some(curr_window_id) = self.curr_window_id {
            let payload = format!(
                "[con_id={}] title_format \"<span> %title </span>\"",
                curr_window_id
            );
            self.run_command(payload).await?;
            self.prev_window_id = self.curr_window_id;
        }
        self.curr_window_id = Some(new_node.id);
        let payload = format!("[con_id={}] title_format \"<b> %title </b>\"", new_node.id);
        self.run_command(payload).await?;
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
impl From<SendError<EventType>> for Error {
    fn from(_: SendError<EventType>) -> Self {
        unimplemented!()
    }
}

pub type Result<T> = std::result::Result<T, Error>;
