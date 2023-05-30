use super::{Error, GenServer, Message};
use std::any::type_name;
use std::thread;

use crate::Mic;

#[derive(Debug)]
#[non_exhaustive]
pub struct ServerHandle<S>
where
	S: GenServer,
{
	pub(crate) thread: thread::JoinHandle<Result<(), Error<S>>>,
	pub(crate) mic: Option<Mic<S::Request, S::StopReason>>,
}

impl<S> ServerHandle<S>
where
	S: GenServer,
{
	pub fn cast(&self, req: S::Request) {
		if let Some(mic) = &self.mic {
			mic.cast(req);
		}
	}

	pub fn mic(&self) -> Result<Mic<S::Request, S::StopReason>, Error<S>> {
		if let Some(mic) = &self.mic {
			Ok(mic.clone())
		} else {
			Err(Error::AlreadyStopped)
		}
	}

	pub fn stop(mut self, reason: S::StopReason) -> Result<(), Error<S>> {
		if let Some(mic) = self.mic {
			self.mic = None;
			if mic.send(Message::Stop(reason)).is_err() {
				log::warn!(
					"unable to send stop message to server, as the thread is already terminated"
				);
			}
			if let Err(e_ref) = self.thread.join() {
				if let Some(e) = e_ref.downcast_ref::<String>() {
					log::warn!("server {} panicked: {e}", type_name::<S>());
				} else {
					log::warn!(
						"server {} panicked with a non-String value",
						type_name::<S>()
					);
				}
			}
			Ok(())
		} else {
			Err(Error::AlreadyStopped)
		}
	}
}
