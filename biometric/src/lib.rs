use autocxx::prelude::*;
use cxx::CxxString;

#[cfg(test)]
mod test;

include_cpp! {
    #include "swift-library.h" // your header file name
    safety!(unsafe) // see details of unsafety policies described in the 'safety' section of the book
    generate!("SwiftLibrary::can_check_biometrics") // add this line for each function or type you wish to generate
    generate!("SwiftLibrary::authenticate")
}

#[must_use]
pub fn can_check_biometrics() -> bool {
    ffi::SwiftLibrary::can_check_biometrics()
}

#[must_use]
pub fn authenticate(localized_reason: String) -> bool {
    ffi::SwiftLibrary::authenticate(localized_reason)
}
