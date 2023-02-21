use std::ffi::CStr;

fn main() {
    env_logger::init();
    println!("Info example");

    let entry = unsafe { ash::Entry::load() }.unwrap_or_else(|e| {
        eprintln!("Failed to load Vulkan: {e}");
        std::process::exit(1);
    });

    let instance_properties = entry
        .enumerate_instance_extension_properties(None)
        .unwrap_or_else(|e| {
            eprintln!("Failed to query for instance extension properties: {e}");
            std::process::exit(1);
        });

    println!(
        "{n} instance extensions supported:",
        n = instance_properties.len()
    );

    for ext in instance_properties.iter() {
        let ext_name = unsafe { CStr::from_ptr(&ext.extension_name as *const i8) };
        println!(r#"  "{name}""#, name = ext_name.to_string_lossy());
    }
}
