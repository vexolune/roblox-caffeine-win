use std::env;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=app.rc");
    println!("cargo:rerun-if-changed=icon.ico");

    let target = env::var("TARGET").unwrap_or_default();
    if target.contains("windows") {
        let out_dir = env::var("OUT_DIR").unwrap();
        let out_file = format!("{}/app.o", out_dir);
        
        let windres = "C:\\msys64\\mingw64\\bin\\windres.exe";
        let status = Command::new(windres)
            .args(&["app.rc", "-o", &out_file])
            .status();

        if let Ok(s) = status {
            if s.success() {
                println!("cargo:rustc-link-arg={}", out_file);
            }
        }
    }
}
