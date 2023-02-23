use faisca::{AppMessage, WindowMessenger, WindowInstance, SafeCString};

fn entry(w: WindowInstance, messenger: WindowMessenger) {
    messenger.send(w, &AppMessage::SetWindowTitle(SafeCString::allocate_from_str("Mamamia").unwrap()));
}

faisca::app_entry!(entry);