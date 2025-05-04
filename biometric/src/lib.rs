use autocxx::prelude::*;

#[cfg(test)]
mod test;

include_cpp! {
    #include "swift-library.h" // your header file name
    // #include "libcxxstdlibshim.h"
    safety!(unsafe_ffi) // see details of unsafety policies described in the 'safety' section of the book
    generate!("SwiftLibrary::can_check_biometrics") // add this line for each function or type you wish to generatex
    // generate!("SwiftLibrary::authenticate")
}

// #[cxx::bridge(namespace = "SwiftLibrary")]
// pub mod ffi2 {
//     unsafe extern "C++" {
//         include!("swift-library.h");
//         type basic_string;
//         fn authenticate(localized_string: UniquePtr<basic_string>) -> bool;
//     }
// }

#[must_use]
pub fn can_check_biometrics() -> bool {
    ffi::SwiftLibrary::can_check_biometrics()
}

// #[must_use]
// pub fn authenticate(localized_reason: String) -> bool {
//     ffi::SwiftLibrary::authenticate(localized_reason.into_cpp())
// }
