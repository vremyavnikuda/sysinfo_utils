#[cfg(target_os = "aix")]
#[path = "aix/mod.rs"]
mod imp;
mod system_info;
mod architecture;
mod system_os;
mod system_version;
mod bit_depth;
mod system_uname;

pub fn get() -> Info {
    impl::current_platform()
}
