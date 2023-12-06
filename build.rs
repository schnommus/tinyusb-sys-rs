use cc::Build;
use std::{env, path::{Path, PathBuf}};
use std::fs::File;
use std::io::prelude::*;

fn add_all_c_files_in_dir(build: &mut Build, path: impl AsRef<Path>) {
    for entry in glob::glob(path.as_ref().join("**/*.c").to_str().unwrap()).unwrap() {
        let path = entry.unwrap();
        if path.extension().and_then(|s| s.to_str()) == Some("c") &&
           !path.to_str().unwrap().contains("portable") {
           build.file(&path);
        }
    }
}

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("Missing OUT_DIR"));

    {
        let mut f = File::create(out_dir.join("tusb_config.h"))
            .expect("Failed to create tusb_config.h");
        f.write_all(include_bytes!("src/tusb_config.h")).ok();

        let mut f = File::create(out_dir.join("stdio.h"))
            .expect("Failed to create tusb_config.h");
        f.write_all(include_bytes!("printf/stdio.h")).ok();
    }

    let stdlib_litex_include_paths = vec![
        concat!(env!("BUILD_DIR"),
                "/software/include"),
        concat!(env!("BUILD_DIR"),
                "/software/libc"),
        concat!(env!("BUILD_DIR"),
                "/../../deps/pythondata-software-picolibc/pythondata_software_picolibc/data/newlib/libc/include"),
        concat!(env!("BUILD_DIR"),
                "/../../deps/litex/litex/soc/cores/cpu/vexriscv"),
        concat!(env!("BUILD_DIR"),
                "/../../deps/litex/litex/soc/software/include"),
    ];

    let common_args = vec![
        "-DCFG_TUSB_DEBUG=3",
        "-march=rv32i2p0_mac",
        "-D__vexriscv__",
        "-no-pie",
        "-Wall",
        "-fno-builtin",
        "-Wstrict-prototypes",
        "-Wold-style-definition",
        "-Wmissing-prototypes",
        "-DCFG_TUSB_MCU=OPT_MCU_LUNA_EPTRI",
        "-fdata-sections",
        "-ffunction-sections",
        "-Os",
    ];

    let clang_args = vec![
        "--target=riscv32-unknown-none-elf",
        "-fvisibility=default",
        "-fshort-enums",
    ];

    let clang_litex_include_args: Vec<String> =
        stdlib_litex_include_paths.iter().map(|s| String::from("-I") + &String::from(*s)).collect();

    let mut build = Build::new();
    add_all_c_files_in_dir(&mut build, "tinyusb/src");
    build.file("luna_eptri/dcd_eptri.c");
    build.file("tinyusb/examples/device/midi_test/src/usb_descriptors.c");
    build.file("printf/printf.c");
    build.file("printf/ctype_.c");

    for flag in common_args.iter() {
        build.flag(flag);
    }

    build
        .include("tinyusb/src")
        .includes(&stdlib_litex_include_paths)
        .include(&out_dir) // for the tusb_config.h file
        .compile("tinyusb");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bindings = bindgen::Builder::default()
        .header("tinyusb/src/tusb.h")
        .rustified_enum(".*")
        .clang_arg(&format!("-I{}", &out_dir.display()))
        .derive_default(true)
        .layout_tests(false)
        .use_core()
        .ctypes_prefix("cty")
        .clang_args(&common_args)
        .clang_args(&clang_args)
        .clang_arg("-Itinyusb/src")
        .clang_args(&clang_litex_include_args)
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Can't write bindings!");
}
