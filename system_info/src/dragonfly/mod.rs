use log::trace;

use crate::{bit_depth, system_os::Type, Info, SystemVersion};

pub fn current_platform() -> Info{
	trace!("dragonfly::current_platform() is called");

	let version = uname("-r")
		.map(SystemVersion::from_string)
		.unwrap_or_else(|| SystemVersion::Unknown);

	let info = Info{
		system_type: Type::DragonFly,
		version,
		bit_depth: bit_depth::get(),
		..Default::default()
	};

	trace!("Returning {:?}", info);
	info
}

#[cfg(test)]
mod tests{
	use super::*;
	use pretty_assertions::assert_eq;
	#[test]
	fn system_type(){
		assert_eq!(current_platform().system_type(), Type::DragonFly);
	}
}