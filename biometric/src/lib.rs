#[cfg(test)]
mod test;

#[swift_bridge::bridge]
mod ffi {
    extern "Rust" {
        #[swift_bridge(Sendable)]
        type CallBackType;
    }
    extern "Swift" {
        #[swift_bridge(Sendable)]
        type CallBackType;
        fn can_check_biometrics() -> bool;
        fn authenticate(localized_reason: &str, callback: CallBackType);
    }
}

struct CallBackType(Box<dyn FnOnce(Result<String, String>)>);

// #[cxx::bridge(namespace = SwiftLibrary)]
// mod ffi {
//     extern "Rust" {
//         type DoThingContext;
//     }
//     unsafe extern "C++" {
//         include!("swift-library.h");
//         fn can_check_biometrics() -> bool;
//         fn authenticate(localized_reason: &CxxString) -> bool;
//         fn authenticate_with_callback(
//             localized_reason: &CxxString,
//             callback: fn(Box<DoThingContext>, ret: bool),
//             ctx: Box<DoThingContext>,
//         );
//     }
// }

// struct DoThingContext(oneshot::Sender<bool>);

// #[must_use]
// pub async fn authenticate_with_callback(localized_reason: &str) -> bool {
//     let (tx, rx) = oneshot::channel();
//     let context = Box::new(DoThingContext(tx));
//     // This is needed to allocate the c++ string on the heap safely
//     let_cxx_string!(stack_pinnned_localized_reason = localized_reason);

//     ffi::authenticate_with_callback(
//         &stack_pinnned_localized_reason,
//         |context, ret| {
//             let _ = context.0.send(ret);
//         },
//         context,
//     );

//     rx.await.unwrap_or_default()
// }

#[must_use = "Need to ensure that biometric capabilities are present before doing anything else."]
pub fn can_check_biometrics() -> bool {
    ffi::can_check_biometrics()
}

// #[must_use]
// pub fn authenticate(localized_reason: &str) -> bool {
//     // This is needed to allocate the c++ string on the heap safely
//     let_cxx_string!(stack_pinnned_localized_reason = localized_reason);
//     ffi::authenticate(&stack_pinnned_localized_reason)
// }
