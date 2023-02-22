mod ffi;

use ffi::{RendererMessage, SafeCString};

#[no_mangle]
extern "C" fn faisca_run_app(message_renderer: extern "C" fn(*const RendererMessage) -> u32) {
    println!("Is it working?");

    let message = RendererMessage::SetWindowTitle(
        SafeCString::allocate_from_str("Glascow Haskell Compiler")
            .unwrap_or_else(|e| {
                eprintln!("Fatal error: {e}");
                std::process::abort();
            })
    );

    println!("Really?");

    message_renderer(&message);
}