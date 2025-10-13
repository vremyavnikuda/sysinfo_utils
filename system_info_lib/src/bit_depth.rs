//src/bit_depth.rs
use std::fmt::Display;
#[cfg(any(
    target_os = "aix",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "linux",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd"
))]
use std::process::{Command, Output};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
/// The bit depth of the system.
pub enum BitDepth {
    /// Unknown bitness (unable to determine).
    Unknown,
    /// 32-bit system.
    X32,
    /// 64-bit system.
    X64,
}

impl Display for BitDepth {
    /// Formats the bit depth as a string.
    ///
    /// Returns a `Result` indicating whether the formatting was successful.
    ///
    /// # Examples
    ///
    ///
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            BitDepth::Unknown => write!(f, "unknown bit depth"),
            BitDepth::X32 => write!(f, "32-bit"),
            BitDepth::X64 => write!(f, "64-bit"),
        }
    }
}

#[cfg(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "linux",
    target_os = "macos",
))]
/// Returns the bit depth of the system as a `BitDepth`.
///
/// The bit depth is determined by running the `getconf LONG_BIT` command
/// and parsing the output. If the output is "32", returns `BitDepth::X32`.
/// If the output is "64", returns `BitDepth::X64`. Otherwise, returns
/// `BitDepth::Unknown`.
pub fn get() -> BitDepth {
    match &Command::new("getconf").arg("LONG_BIT").output() {
        Ok(Output { stdout, .. }) if stdout == b"32\n" => BitDepth::X32,
        Ok(Output { stdout, .. }) if stdout == b"64\n" => BitDepth::X64,
        _ => BitDepth::Unknown,
    }
}

/// Returns the bit depth of the system as a `BitDepth`.
///
/// The bit depth is determined by running the `sysctl -n hw.machine_arch` command
/// and checking the output. If the output is "amd64\n", "x86_64\n", "aarch64\n",
/// or "sparc64\n", the bit depth is `BitDepth::X64`. If the output is "i386\n"
/// or "earmv7hf\n", the bit depth is `BitDepth::X32`. Otherwise, the bit depth is
/// `BitDepth::Unknown`.
///
/// This function is only available on NetBSD systems.
#[cfg(target_os = "netbsd")]
pub fn get() -> BitDepth {
    match &Command::new("sysctl")
        .arg("-n")
        .arg("hw.machine_arch")
        .output()
    {
        Ok(Output { stdout, .. }) if stdout == b"amd64\n" => BitDepth::X64,
        Ok(Output { stdout, .. }) if stdout == b"x86_64\n" => BitDepth::X64,
        Ok(Output { stdout, .. }) if stdout == b"i386\n" => BitDepth::X32,
        Ok(Output { stdout, .. }) if stdout == b"aarch64\n" => BitDepth::X64,
        Ok(Output { stdout, .. }) if stdout == b"earmv7hf\n" => BitDepth::X32,
        Ok(Output { stdout, .. }) if stdout == b"sparc64\n" => BitDepth::X64,
        _ => BitDepth::Unknown,
    }
}

/// Returns the bit depth of the system as a `BitDepth`.
///
/// The bit depth is determined by running the `isainfo -b` command and
/// checking the output. If the output is "64\n", the bit depth is
/// `BitDepth::X64`. If the output is "32\n", the bit depth is
/// `BitDepth::X32`. Otherwise, the bit depth is `BitDepth::Unknown`.
///
/// This function is only available on Illumos systems.
#[cfg(target_os = "illumos")]
pub fn get() -> BitDepth {
    match Command::new("isainfo").arg("-b").output() {
        Ok(Output { stdout, .. }) if stdout == b"64\n" => BitDepth::X64,
        Ok(Output { stdout, .. }) if stdout == b"32\n" => BitDepth::X32,
        _ => BitDepth::Unknown,
    }
}

#[cfg(target_os = "openbsd")]
pub fn get() -> BitDepth {
    match &Command::new("sysctl").arg("-n").arg("hw.machine").output() {
        Ok(Output { stdout, .. }) if stdout == b"amd64\n" => BitDepth::X64,
        Ok(Output { stdout, .. }) if stdout == b"x86_64\n" => BitDepth::X64,
        Ok(Output { stdout, .. }) if stdout == b"i386\n" => BitDepth::X32,
        Ok(Output { stdout, .. }) if stdout == b"aarch64\n" => BitDepth::X64,
        Ok(Output { stdout, .. }) if stdout == b"earmv7hf\n" => BitDepth::X32,
        Ok(Output { stdout, .. }) if stdout == b"sparc64\n" => BitDepth::X64,
        _ => BitDepth::Unknown,
    }
}
/// Returns the bit depth of the system as a `BitDepth`.
///
/// The bit depth is determined by running the `prtconf -c` command and
/// checking the output. If the output is "CPU :64-bit\n", the bit depth
/// is `BitDepth::X64`. If the output is "CPU :32-bit\n", the bit depth
/// is `BitDepth::X32`. Otherwise, the bit depth is `BitDepth::Unknown`.
///
/// This function is only available on AIX systems.
#[cfg(target_os = "aix")]
pub fn get() -> BitDepth {
    match Command::new("prtconf").arg("-c").output() {
        Ok(Output { stdout, .. }) if stdout == b"CPU :64-bit\n" => BitDepth::X64,
        Ok(Output { stdout, .. }) if stdout == b"CPU :32-bit\n" => BitDepth::X32,
        _ => BitDepth::Unknown,
    }
}

#[cfg(all(
    test,
    any(
        target_os = "aix",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "linux",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd"
    )
))]
mod bit_depth_tests {
    use super::*;
    use pretty_assertions::assert_ne;

    /// Tests the `get` function to ensure it returns a bit depth
    /// that is not `BitDepth::Unknown`.
    #[test]
    fn get_bitness() {
        let b = get();
        assert_ne!(b, BitDepth::Unknown);
    }

    /// Tests the `Display` implementation for the `BitDepth` enum.
    ///
    /// This test verifies that each variant of `BitDepth` is correctly
    /// formatted as a string according to its expected representation.
    /// - `BitDepth::Unknown` should be represented as "unknown bit depth".
    /// - `BitDepth::X32` should be represented as "32-bit".
    /// - `BitDepth::X64` should be represented as "64-bit".
    #[test]
    fn display() {
        let data = [
            (BitDepth::Unknown, "unknown bit depth"),
            (BitDepth::X32, "32-bit"),
            (BitDepth::X64, "64-bit"),
        ];

        for (bit_depth, expected) in &data {
            assert_eq!(&bit_depth.to_string(), expected);
        }
    }
}
