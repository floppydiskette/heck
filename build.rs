use gtk::gio;

fn main() {
    gio::compile_resources(
        "resources",
        "resources/res.gresource.xml",
        "ct.gresource",
    );

    // run the compile_settings_schema.bat script
    // this script will function on both windows and linux
    // it will compile the settings schema and place it in the correct directory
    if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .args(&["/C", "compile_settings_schema.bat"])
            .spawn()
            .expect("failed to run compile_settings_schema.bat");
    } else {
        std::process::Command::new("sh")
            .arg("compile_settings_schema.bat")
            .spawn()
            .expect("failed to run compile_settings_schema.bat");
    }
}