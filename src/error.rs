use thiserror::Error;

use super::GenServer;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error<S: GenServer> {
	#[error("a server-specific error occurred: {0}")]
	Server(S::Error),

	#[error("failed to spawn GenServer thread")]
	ThreadSpawn,

	#[error("GenServer thread panicked: {0}")]
	ThreadPanic(String),

	#[error("request made to already stopped server")]
	AlreadyStopped,

	#[error("all senders have terminated")]
	NoSenders,
}
