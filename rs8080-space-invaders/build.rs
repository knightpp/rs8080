fn main() {
    #[cfg(feature = "sound")]
    {
        use std::env::var;
        use std::fs;
        use std::path::PathBuf;
        let target_dir = find_cargo_target_dir();
        let manifest_dir = PathBuf::from(var("CARGO_MANIFEST_DIR").unwrap());

        // I copied this function from `sdl2-sys` build script.
        fn find_cargo_target_dir() -> PathBuf {
            // Infer the top level cargo target dir from the OUT_DIR by searching
            // upwards until we get to $CARGO_TARGET_DIR/build/ (which is always one
            // level up from the deepest directory containing our package name)
            let pkg_name = var("CARGO_PKG_NAME").unwrap();
            let mut out_dir = PathBuf::from(var("OUT_DIR").unwrap());
            loop {
                {
                    let final_path_segment = out_dir.file_name().unwrap();
                    if final_path_segment.to_string_lossy().contains(&pkg_name) {
                        break;
                    }
                }
                if !out_dir.pop() {
                    panic!("Malformed build path: {}", out_dir.to_string_lossy());
                }
            }
            out_dir.pop();
            out_dir.pop();
            out_dir
        }

        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        {
            let dll_path = manifest_dir.join("SDL2_mixer\\64\\SDL2_mixer.dll");
            let new_dll_path = target_dir.join("SDL2_mixer.dll");
            println!(
                r"cargo:rustc-link-search={}\SDL2_mixer\64\",
                manifest_dir.display()
            );

            fs::copy(&dll_path, &new_dll_path).expect(&format!(
                "Failed to copy SDL2_mixer.dll: from {:?} to {:?}",
                dll_path, target_dir
            ));
        }

        #[cfg(all(target_os = "windows", target_arch = "x86"))]
        {
            let dll_path = manifest_dir.join("SDL2_mixer\\86\\SDL2_mixer.dll");
            let new_dll_path = target_dir.join("SDL2_mixer.dll");
            println!(
                r"cargo:rustc-link-search={}\SDL2_mixer\86\",
                manifest_dir.display()
            );

            fs::copy(&dll_path, &new_dll_path).expect(&format!(
                "Failed to copy SDL2_mixer.dll: from {:?} to {:?}",
                dll_path, target_dir
            ));
        }
    }
}
