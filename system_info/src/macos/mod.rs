use std::process::Command;

use log::{trace, warn};

use crate::{system_info::Info, system_os::Type,  SystemMatcher, SystemVersion};

pub fn current_platform() -> Info{
    trace!("macos::current_platform() is called");
    
    let info = Info{
        system_type: Type::Macos,
        version: version(),
        ..Default::default()

    };
    trace!("Returning {:?}", info);
    info
}

fn version () -> SystemVersion{
    match product_version(){
        None => SystemVersion::Unknown,
        Some(value) => SystemVersion::from_string(value),
    } 
}

fn product_version() -> Option<String>{
    match Command::new("sw_vers").output() {
        Ok(value)=>{
            let output = String::from_utf8_lossy(&value.stdout);
            trace!("sw_vers output returned: {:?}",output);
            parce(&output)
        }
        Err(e) => {
            warn!("Failed to run sw_vers: {:?}", e);
            None
        }
    }
}

fn parce (sw_vers_output: &str) -> Option<String>{
    SystemMatcher::PrefixedVersion { prefix:"ProductVersion:" }.find(sw_vers_output)
}

#[cfg(test)]
mod macos_tests{
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
   fn system_type(){
       let version = current_platform();
       assert_eq!(Type::Macos, version.system_type());
   }

   #[test]
   fn system_version(){
       let version = version();
       assert_eq!(SystemVersion::Unknown, version);
   }

   #[test]
   fn product_version_test(){
        let version = product_version();
        assert!(version.is_some());
   }

   #[test]
   fn parce_version_macos(){
        let parce_version_macos_output = parce(sw_vers_output());
        assert!(parce_version_macos_output.is_some());
   }
   
   fn sw_vers_output() -> &'static str {
        "ProductName:	Mac OS X\n\
        ProductVersion:	10.15.7\n\
        BuildVersion:	19H15"
   }

   #[test]
   fn parce_beta_version_macos(){
        let parce_output_macos = parce(sw_vers_output_beta());
        assert!(parce_output_macos.is_none());
   }

   fn sw_vers_output_beta()-> &'static str{
         "ProductName:	Mac OS X\n\
         ProductVersion:	11.0.1\n\
         BuildVersion:	20B29"
   }

   #[test]
   fn parce_double_digit_patch_version(){
        let parce_output_macos = parce(sw_vers_output_double_digit_patch_version());
        assert!(parce_output_macos.is_some());
   }
   
   fn sw_vers_output_double_digit_patch_version() -> &'static str {
        "ProductName:	Mac OS X\n\
        ProductVersion:	10.15.21\n\
        BuildVersion:	ABCD123"
    }
}