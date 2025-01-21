use crate::bit_depth::BitDepth;
use crate::system_os::Type;
use crate::system_version::SystemVersion;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Info {
    pub(crate) system_type: Type,
    pub(crate) version: SystemVersion,
    pub(crate) edition: Option<String>,
    pub(crate) codename: Option<String>,
    pub(crate) bit_depth: BitDepth,
    pub(crate) architecture: Option<String>,
}



impl Info {
    /// Returns an `Info` instance with all fields set to their unknown or default values.
    ///
    /// # Returns
    ///
    /// * `Info` - An instance of `Info` with fields:
    ///     - `system_type`: `Type::Unknown`
    ///     - `version`: `Version::unknown()`
    ///     - `edition`: `None`
    ///     - `codename`: `None`
    ///     - `bit_depth`: `BitDepth::Unknown`
    ///     - `architecture`: `None`
    pub fn unknown() -> Self {
        Self {
            system_type: Type::Unknown,
            version: SystemVersion::Unknown,
            edition: None,
            codename: None,
            bit_depth: BitDepth::Unknown,
            architecture: None,
        }
    }

    pub fn with_type(system_type: Type) -> Self {
        Self {
            system_type,
            ..Default::default()
        }
    }
    /// Returns the system type.
    ///
    /// # Returns
    /// Returns the version of the OS.
    ///
    /// # Returns
    ///
    /// * `&Version` - The version of the OS.
    ///
    /// * `Type` - The system type.
    pub fn system_type(&self) -> Type {
        self.system_type
    }

    /// Returns the version of the OS.
    ///
    /// # Returns
    ///
    /// * `&Version` - The version of the OS.
    pub fn version(&self) -> &SystemVersion {
        &self.version
    }

    /// Returns the edition of the OS.
    ///
    /// # Returns
    ///
    /// * `Option<&str>` - The edition of the OS, if known.
    pub fn edition(&self) -> Option<&str> {
        self.edition.as_ref().map(String::as_ref)
    }

    /// Returns the codename of the OS.
    ///
    /// # Returns
    ///
    /// * `Option<&str>` - The codename of the OS, if known.
    pub fn codename(&self) -> Option<&str> {
        self.codename.as_ref().map(String::as_ref)
    }

    /// Returns the bit depth of the OS.
    ///
    /// # Returns
    ///
    /// * `BitDepth` - The bit depth of the OS.
    pub fn bit_depth(&self) -> BitDepth {
        self.bit_depth
    }

    /// Returns the architecture of the OS.
    ///
    /// # Returns
    ///
    /// * `Option<&str>` - The architecture of the OS, if known.
    pub fn architecture(&self) -> Option<&str> {
        self.architecture.as_ref().map(String::as_ref)
    }
}
impl Default for Info {
    /// Creates a new `Info` instance with all fields set to their unknown or default values.
    ///
    /// # Returns
    ///
    /// * `Info` - An instance of `Info` with fields:
    ///     - `system_type`: `Type::Unknown`
    ///     - `version`: `Version::unknown()`
    ///     - `edition`: `None`
    ///     - `codename`: `None`
    ///     - `bit_depth`: `BitDepth::Unknown`
    ///     - `architecture`: `None`
    fn default() -> Self {
        Self::unknown()
    }
}

impl Display for Info {
    /// Formats the value using the given formatter.
    ///
    /// The format is as follows:
    ///
    /// `<system_type> [<edition>] (<codename>) <version>, <bit_depth>, <architecture>`
    ///
    /// Where:
    ///
    /// - `<system_type>` is the type of the operating system
    /// - `<edition>` is the edition of the operating system, if known
    /// - `<codename>` is the codename of the operating system, if known
    /// - `<version>` is the version of the operating system
    /// - `<bit_depth>` is the bit depth of the operating system, if known
    /// - `<architecture>` is the architecture of the operating system, if known
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.system_type)?;
        if let Some(edition) = &self.edition {
            write!(f, " {}", edition)?;
        }
        if let Some(codename) = &self.codename {
            write!(f, " ({})", codename)?;
        }
        write!(f, " {}", self.version)?;
        if self.bit_depth != BitDepth::Unknown {
            write!(f, ", {}", self.bit_depth)?;
        }
        if let Some(architecture) = &self.architecture {
            write!(f, ", {}", architecture)?;
        }
        Ok(())
    }
}
