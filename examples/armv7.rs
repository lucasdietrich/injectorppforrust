use injectorpp::interface::injector::*;
use std::fs;


fn try_repair() -> Result<(), String> {
    if let Err(e) = fs::create_dir_all("/tmp/target_files") {
        println!("Failed to create directory: {}", e);
        return Err(format!("Could not create directory: {}", e));
    }

    println!("Directory created successfully.");

    Ok(())
}

fn fake_create_dir_all(path: &str) -> std::io::Result<()> {
    println!("Fake create_dir_all called with path: {}", path);
    Ok(())
}

fn main() {
    println!("This is an example of a Rust program for ARMv7 architecture.");

    let mut injector = InjectorPP::new();
    injector
        .when_called(
            injectorpp::func!(fn (fs::create_dir_all)(&'static str) -> std::io::Result<()>),
        )
        // .will_execute(injectorpp::fake!(
        //     func_type: fn(path: &str) -> std::io::Result<()>,
        //     when: path == "/tmp/target_files",
        //     returns: Ok(()),
        //     times: 1
        // ));
        .will_execute_raw(injectorpp::func!(fn (fake_create_dir_all)(&'static str) -> std::io::Result<()>));

    assert!(try_repair().is_ok());
}
