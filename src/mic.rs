use crossbeam_channel::{SendError, Sender};

use super::Message;

#[derive(Debug)]
pub struct Mic<R, SR> {
	pub(crate) tx: Sender<Message<R, SR>>,
}

impl<R, SR> Clone for Mic<R, SR> {
	fn clone(&self) -> Self {
		Self {
			tx: self.tx.clone(),
		}
	}
}

impl<R, SR> Mic<R, SR> {
	pub fn cast(&self, req: R) {
		if self.tx.send(Message::Cast(req)).is_err() {
			log::warn!("cast sent to closed channel");
		}
	}

	pub(crate) fn send(&self, msg: Message<R, SR>) -> Result<(), SendError<Message<R, SR>>> {
		self.tx.send(msg)
	}
}
