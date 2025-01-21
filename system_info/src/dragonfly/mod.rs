use std::process::Command;
use log::trace;
use crate::{system_version, Info};

pub fn current_platform() -> Info{
	trace!("dragonfly::current_platform() is called");

	let version = uname()
}
