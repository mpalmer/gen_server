mod error;
mod mic;
mod server_handle;

pub use error::Error;
pub use mic::Mic;
pub use server_handle::ServerHandle;

use std::any::type_name;

pub trait GenServer: Sized + std::fmt::Debug {
	type Args;
	type Error: std::error::Error + Send;
	type Request: std::fmt::Debug;
	type StopReason: std::fmt::Debug;

	fn init(args: Self::Args) -> Result<Self, Self::Error>;

	fn terminate(&mut self, reason: Result<Self::StopReason, Self::Error>) {
		log::debug!("Terminating because {:?}", reason);
	}

	//fn handle_call<R>(&mut self, request: Request) -> Result<(R, E>;

	fn handle_cast(&mut self, request: Self::Request) -> Result<Status<Self>, Self::Error>;

	//handle_continue/2
	//handle_info/2
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Status<S: GenServer> {
	Continue,
	Stop(S::StopReason),
}

#[derive(Debug)]
enum Message<R, SR> {
	Cast(R),
	Stop(SR),
}

use crossbeam_channel::{unbounded as new_unbounded_channel, Receiver};

use std::thread;

pub fn start<S>(args: S::Args) -> Result<ServerHandle<S>, Error<S>>
where
	S: GenServer + Send + 'static,
	<S as GenServer>::Request: Send,
	<S as GenServer>::StopReason: Send,
{
	let server = S::init(args).map_err(|e| Error::Server(e))?;

	let (tx, rx) = new_unbounded_channel();
	#[allow(clippy::map_err_ignore)] // This error provides no extra information
	let joiner = thread::Builder::new()
		.name(format!("GenServer<{}>", type_name::<S>()))
		.spawn(move || message_loop(&rx, server))
		.map_err(|_| Error::ThreadSpawn)?;

	Ok(ServerHandle {
		thread: joiner,
		mic: Some(Mic { tx }),
	})
}

fn message_loop<S: GenServer>(
	rx: &Receiver<Message<S::Request, S::StopReason>>,
	mut server: S,
) -> Result<(), Error<S>> {
	loop {
		#[allow(clippy::map_err_ignore)] // This error provides no extra information
		let msg = rx.recv().map_err(|_| Error::NoSenders)?;
		log::trace!("Received {msg:?}");

		let result = match msg {
			Message::Cast(request) => server.handle_cast(request),
			Message::Stop(reason) => {
				log::debug!("Stopping by request");
				Ok(Status::Stop(reason))
			}
		};

		match result {
			Ok(Status::Continue) => (),
			Ok(Status::Stop(reason)) => break server.terminate(Ok(reason)),
			Err(e) => break server.terminate(Err(e)),
		}
	}

	Ok(())
}
