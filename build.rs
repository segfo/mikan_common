use std::path::Path;
use std::process::Command;
use std::{env, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let srcs = ["io.s"];
    let lib_name = "myffi";
    for i in 0..srcs.len() {
        let src = Path::new(srcs[i]).file_stem().unwrap().to_str().unwrap();
        Command::new("gcc")
            .args(&[
                &format!("./src/hardware/x86_64/{}.s", src),
                "-c",
                "-fPIC",
                "-o",
            ])
            .arg(&format!("{}/{}_kernel.o", out_dir, src))
            .status()
            .unwrap();
        Command::new("x86_64-w64-mingw32-gcc")
            .args(&[
                &format!("src/hardware/x86_64/{}.s", src),
                "-c",
                "-fPIC",
                "-o",
            ])
            .arg(&format!("{}/{}_loader.o", out_dir, src))
            .status()
            .unwrap();
        Command::new("ar")
            .args(&[
                "crUs",
                &format!("lib{}.a", lib_name),
                &format!("{}_kernel.o", src),
            ])
            .current_dir(&Path::new(&out_dir))
            .status()
            .unwrap();
        Command::new("x86_64-w64-mingw32-ar")
            .args(&[
                "crUs",
                &format!("{}.lib", lib_name),
                &format!("{}_loader.o", src),
            ])
            .current_dir(&Path::new(&out_dir))
            .status()
            .unwrap();
    }
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static={}", lib_name);
    Ok(())
}