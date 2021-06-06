use log::{error, info, warn};
use std::error::Error as StdError;
use std::fmt;
use tokio_i3ipc::event::{Event as I3Event, Subscribe, WindowChange};
use tokio_i3ipc::reply::Node;
use tokio_i3ipc::I3;

pub struct I3Manager {
    prev_window_id: Option<usize>,
    curr_window_id: Option<usize>,
    i3: I3,
}

impl I3Manager {
    pub async fn new() -> Result<I3Manager> {
        let i3 = I3::connect().await?;
        Ok(Self {
            prev_window_id: None,
            curr_window_id: None,
            i3,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        self.i3.subscribe([Subscribe::Window]).await?;

        loop {
            tokio::select! {
                i3_event = self.i3.read_event() => {
                    match i3_event {
                        Ok(i3_event) => self.handle_event(i3_event).await?,
                        Err(err) => error!("Got i3 error: {:#?}", err)
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_event(&mut self, event: I3Event) -> Result<()> {
        // Can't pattern match on a box in stable rust (june 2021)
        // https://doc.rust-lang.org/stable/unstable-book/language-features/box-patterns.html
        if let I3Event::Window(event) = event {
            if event.change == WindowChange::Focus {
                self.focus_change(event.container).await?
            }
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

pub type Result<T> = std::result::Result<T, Error>;
