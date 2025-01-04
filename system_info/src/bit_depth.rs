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
use std::process::Command;
use std::process::Output;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum BitDepth {
    Unknown,
    X32,
    X64,
}

impl Display for BitDepth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            BitDepth::Unknown => write!(f, "Unknown"),
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

pub fn get() -> BitDepth {
    match &Command::new("getconf").arg("LONG_BIT").output() {
        Ok(Output { stdout, .. }) if stdout == b"32\n" => BitDepth::X32,
        Ok(Output { stdout, .. }) if stdout == b"64\n" => BitDepth::X64,
        _ => BitDepth::Unknown,
    }
}

#[cfg(target_os = "netbsd")]
pub fn get() -> BitDepth {
    match &Command::new("sysctl")
        .arg("-n")
        .arg("hw.machine_arch")
        .output()
    {
        Ok(Output { stdout, .. }) if stdout == b"amd64\n" => Bitness::X64,
        Ok(Output { stdout, .. }) if stdout == b"x86_64\n" => Bitness::X64,
        Ok(Output { stdout, .. }) if stdout == b"i386\n" => Bitness::X32,
        Ok(Output { stdout, .. }) if stdout == b"aarch64\n" => Bitness::X64,
        Ok(Output { stdout, .. }) if stdout == b"earmv7hf\n" => Bitness::X32,
        Ok(Output { stdout, .. }) if stdout == b"sparc64\n" => Bitness::X64,
        _ => Bitness::Unknown,
    }
}

#[cfg(target_os = "illumos")]
pub fn get() -> BitDepth {
    match Command::new("isainfo").arg("-b").output() {
        Ok(Output { stdout, .. }) if stdout == b"64\n" => Bitness::X64,
        Ok(Output { stdout, .. }) if stdout == b"32\n" => Bitness::X32,
        _ => Bitness::Unknown,
    }
}

#[cfg(target_os = "aix")]
pub fn get() -> BitDepth {
    match Command::new("prtconf").arg("-c").output() {
        Ok(Output { stdout, .. }) if stdout == b"CPU :64-bit\n" => Bitness::X64,
        Ok(Output { stdout, .. }) if stdout == b"CPU :32-bit\n" => Bitness::X32,
        _ => Bitness::Unknown,
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

mod tests {
    use super::*;
    use pretty_assertions::assert_ne;

    #[test]
    fn get_bitness() {
        let b = get();
        assert_ne!(b, BitDepth::Unknown);
    }

    #[test]
    fn display() {
        let data = [
            (BitDepth::Unknown, "unknown bit depth"),
            (BitDepth::X32, "32-bit"),
            (BitDepth::X64, "64-bit"),
        ];

        for (bitness, expected) in &data {
            assert_eq!(&bitness.to_string(), expected);
        }
    }
}
