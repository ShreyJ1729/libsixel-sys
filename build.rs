use std::env;
use std::path::PathBuf;

fn main() {
    // 1. Build C libraries
    let commands = [
        "cd libsixel && ./configure --enable-python=no && make",
        "cd libpng && ./configure && make",
        "cd libjpeg-turbo && cmake -G\"Unix Makefiles\" && make",
    ];
    for command in commands.iter() {
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .expect("Failed to execute command");
        println!("{}", String::from_utf8_lossy(&output.stdout));
        println!("{}", String::from_utf8_lossy(&output.stderr));
    }

    // 2. Create bindings to libsixel

    // Generate the bindings
    let mut bindings = bindgen::Builder::default().header("libsixel/include/sixel.h");

    // iterate over sixel.h and add all enum names to the builder
    for line in include_str!("libsixel/include/sixel.h").lines() {
        let trimmed = line.trim().split_whitespace().collect::<Vec<_>>();
        if trimmed.len() < 2 {
            continue;
        }
        if trimmed[0] == "enum" {
            bindings = bindings.rustified_enum(trimmed[1]);
            println!("added enum {}", trimmed[1]);
        } else if trimmed[0] == "typedef" && trimmed[1] == "enum" {
            bindings = bindings.rustified_enum(trimmed[2]);
            println!("added enum {}", trimmed[2]);
        }
    }
    // Create bindings and write to src/generated_bindings.rs
    let out_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    bindings
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path.join("src/generated_bindings.rs"))
        .expect("Couldn't write bindings!");

    // 3. Compile and link the C libraries
    let mut build = cc::Build::new();

    // Tell cargo to tell rustc to link the libsixel submodule directory
    println!("cargo:rustc-link-lib=static=libsixel");
    // Tell cargo where to find compiled C library
    println!("cargo:rustc-link-search=native={}", out_path.display());

    // Include assorted .h files (libpng and libjpeg are deps)
    build.include("libsixel");
    build.include("libsixel/include");
    build.include("libjpeg-turbo");
    build.include("libpng");

    // Add all .c files in libsixel/src to the build
    for entry in glob::glob("libsixel/src/*.c").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => build.file(path),
            Err(e) => panic!("Error reading file: {}", e),
        };
    }

    build.compile("libsixel");
}
