use autocxx::prelude::*; // use all the main autocxx functions

#[cfg(test)]
mod test;

include_cpp! {
    #include "swift-library.h" // your header file name
    safety!(unsafe) // see details of unsafety policies described in the 'safety' section of the book
    generate!("SwiftLibrary::can_check_biometrics") // add this line for each function or type you wish to generate
}

#[must_use]
pub fn can_check_biometrics() -> bool {
    ffi::SwiftLibrary::can_check_biometrics()
}
