use autocxx::prelude::*;

#[cfg(test)]
mod test;

include_cpp! {
    #include "swift-library.h" // your header file name
    safety!(unsafe) // see details of unsafety policies described in the 'safety' section of the book
    generate!("SwiftLibrary::can_check_biometrics") // add this line for each function or type you wish to generatex
    generate!("SwiftLibrary::authenticate")
}

#[must_use]
pub fn can_check_biometrics() -> bool {
    ffi::SwiftLibrary::can_check_biometrics()
}

#[must_use]
pub fn authenticate(localized_reason: &str) -> bool {
    unsafe {
        // This is actually safe because we are using a C++ function that is safe
        // as well as we know that the string would always have a maximum length of 255 characters.
        #![allow(clippy::cast_possible_truncation)]
        ffi::SwiftLibrary::authenticate(localized_reason.as_ptr(), localized_reason.len() as u8)
    }
}
