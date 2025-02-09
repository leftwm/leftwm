use crate::SmithayWindowHandle;
use std::{
    cell::RefCell,
    sync::mpsc::{self, Receiver, SendError, Sender},
};
use tokio::sync::mpsc::{
    self as async_mpsc, error::TrySendError, Receiver as AsyncReceiver, Sender as AsyncSender,
};

use leftwm_core::DisplayEvent;

pub struct EventChannelSender {
    event_sender: Sender<DisplayEvent<SmithayWindowHandle>>,
    event_notify_sender: AsyncSender<()>,
}

impl EventChannelSender {
    pub fn send_event(
        &self,
        event: DisplayEvent<SmithayWindowHandle>,
    ) -> Result<(), SendError<()>> {
        self.event_sender.send(event).map_err(|_| SendError(()))?;
        // If this returns a TrySendError::Full we ignore that since that means that the other side
        // is already notified but not yet collected.
        if let Err(TrySendError::Closed(_)) = self.event_notify_sender.try_send(()) {
            return Err(SendError(()));
        }

        Ok(())
    }
}

pub struct EventChannelReceiver {
    event_receiver: Receiver<DisplayEvent<SmithayWindowHandle>>,
    // We wanna get mut access to this for recieving without self being mut because
    // `DisplayServer::wait_readable` doesn't give us mutable access to self.
    event_notify_receiver: RefCell<AsyncReceiver<()>>,
}

impl EventChannelReceiver {
    pub async fn wait_readable(&self) {
        self.event_notify_receiver
            .borrow_mut()
            .recv()
            .await
            .unwrap()
    }

    pub fn collect_events(&mut self) -> Vec<DisplayEvent<SmithayWindowHandle>> {
        self.event_receiver.try_iter().collect()
    }
}

pub fn event_channel() -> (EventChannelSender, EventChannelReceiver) {
    let (event_sender, event_receiver) = mpsc::channel();
    let (event_notify_sender, event_notify_receiver) = async_mpsc::channel(1);
    (
        EventChannelSender {
            event_sender,
            event_notify_sender,
        },
        EventChannelReceiver {
            event_receiver,
            event_notify_receiver: RefCell::new(event_notify_receiver),
        },
    )
}
