use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    println!("cargo:rerun-if-changed=assets/app.rc");

    // Set main thread stack size to 8 MB via linker flag.
    // LunulaApp is a large struct initialized on the main thread stack;
    // the default 1 MB Windows stack is insufficient and causes stack overflow.
    // eframe/winit requires event loop on the main OS thread, so we cannot
    // use thread::Builder::stack_size() workaround — linker flag is the correct fix.
    #[cfg(windows)]
    println!("cargo:rustc-link-arg=-Wl,--stack,8388608"); // 8 MB — GNU ld (MinGW) stack flag

    #[cfg(windows)]
    {
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let assets_dir = manifest_dir.join("assets");

        let resource_o = out_dir.join("resource.o");

        // Compile .rc into .o using windres inside assets directory to avoid space in path issues
        let windres_status = Command::new("windres.exe")
            .current_dir(&assets_dir)
            .arg("-i")
            .arg("app.rc")
            .arg("-o")
            .arg(&resource_o)
            .status();

        if let Ok(status) = windres_status {
            if status.success() {
                // Directly pass compiled object file to linker to prevent GNU ld from discarding unreferenced .rsrc
                println!("cargo:rustc-link-arg={}", resource_o.display());
            }
        }
    }
}
