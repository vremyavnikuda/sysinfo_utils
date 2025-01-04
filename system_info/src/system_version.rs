use std::fmt::Display;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Operating system version.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SystemVersion {
    Unknown,
    Semantic(u64, u64, u64),
    Rolling(Option<String>),
    Custom(String),
}

impl SystemVersion {
    pub fn from_string<S: Into<String> + AsRef<str>>(s: S) -> Self {
        if s.as_ref().is_empty() {
            Self::Unknown
        } else if let Some((major, minor, patch)) = parse_version(s.as_ref()) {
            Self::Semantic(major, minor, patch)
        } else {
            Self::Custom(s.into())
        }
    }
}

impl Default for SystemVersion {
    fn default() -> Self {
        SystemVersion::Unknown
    }
}

impl Display for SystemVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            SystemVersion::Unknown => write!(f, "Unknown"),
            SystemVersion::Semantic(major, minor, patch) => {
                write!(f, "{}.{}.{}", major, minor, patch)
            }
            SystemVersion::Rolling(ref codename) => {
                if let &Some(ref codename) = codename {
                    write!(f, "Rolling ({})", codename)
                } else {
                    write!(f, "Rolling")
                }
            }
            SystemVersion::Custom(ref version) => write!(f, "{}", version),
        }
    }
}

fn parse_version(s: &str) -> Option<(u64, u64, u64)> {
    let mut parts = s.trim().split_terminator('.').fuse();

    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;

    if parts.next().is_some() {
        return None;
    }

    Some((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_string_empty() {
        let version = SystemVersion::from_string("");
        assert_eq!(version, SystemVersion::Unknown);
    }

    #[test]
    fn test_from_string_semantic() {
        let version = SystemVersion::from_string("1.2.3");
        assert_eq!(version, SystemVersion::Semantic(1, 2, 3));
    }

    #[test]
    fn test_from_string_custom() {
        let version = SystemVersion::from_string("custom_version");
        assert_eq!(version, SystemVersion::Custom("custom_version".to_string()));
    }

    #[test]
    fn test_display_unknown() {
        let version = SystemVersion::Unknown;
        assert_eq!(version.to_string(), "Unknown");
    }

    #[test]
    fn test_display_semantic() {
        let version = SystemVersion::Semantic(1, 2, 3);
        assert_eq!(version.to_string(), "1.2.3");
    }

    #[test]
    fn test_display_rolling_with_codename() {
        let version = SystemVersion::Rolling(Some("codename".to_string()));
        assert_eq!(version.to_string(), "Rolling (codename)");
    }

    #[test]
    fn test_display_rolling_without_codename() {
        let version = SystemVersion::Rolling(None);
        assert_eq!(version.to_string(), "Rolling");
    }

    #[test]
    fn test_display_custom() {
        let version = SystemVersion::Custom("custom_version".to_string());
        assert_eq!(version.to_string(), "custom_version");
    }

    #[test]
    fn test_parse_version_valid() {
        let parsed = parse_version("1.2.3");
        assert_eq!(parsed, Some((1, 2, 3)));
    }

    #[test]
    fn test_parse_version_invalid() {
        let parsed = parse_version("1.2");
        assert_eq!(parsed, None);
    }
}
