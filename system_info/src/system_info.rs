use std::fmt::Display;
use crate::system_os::Type;
use crate::system_version::SystemVersion;

pub struct Info {
    pub(crate) system_type: Type,
    pub(crate) version: SystemVersion,
    pub(crate) edition: Option<String>,
    pub(crate) codename: Option<String>,
    pub(crate) bit_depth: BitDepth,
    pub(crate) architecture: Option<String>,
}

impl Info {
    pub fn unknown() -> Self {
        Self {
            system_type: Type::Unknown,
            version: Version::unknown(),
            edition: None,
            codename: None,
            bit_depth: BitDepth::Unknown,
            architecture: None,
        }
    }

    pub fn write_type(system_type: Type) -> Self {
        Self {
            system_type,
            ..Default::default()
        }
    }

    pub fn system_type(&self) -> Type {
        self.system_type
    }

    pub fn version(&self) -> &Version {
        self.version
    }

    pub fn edition(&self) -> Option<&str> {
        self.edition.as_ref().map(String::as_ref)
    }

    pub fn codename(&self) -> Option<&str> {
        self.codename.as_ref().map(String::as_ref)
    }

    pub fn bit_depth(&self) -> BitDepth {
        self.bit_depth
    }

    pub fn architecture(&self) -> Option<&str> {
        self.architecture.as_ref().map(String::as_ref)
    }
}
impl Default for Info {
    fn default() -> Self {
        Self::unknown()
    }
}

impl Display for Info {
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