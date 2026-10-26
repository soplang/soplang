use std::process::Command;
use std::{env, path::PathBuf};

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("Cargo did not set target OS");
    if target_os == "windows" {
        // Parsing and JIT-compiling a complete web module can use more than
        // the PE image's default 1 MiB main-thread stack.
        let argument = if env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
            "/STACK:8388608"
        } else {
            "-Wl,--stack,8388608"
        };
        println!("cargo:rustc-link-arg-bin=soplang={argument}");
    }
    if std::env::var_os("CARGO_FEATURE_EMBEDDED_LLVM").is_none() {
        return;
    }
    println!("cargo:rerun-if-env-changed=SOPLANG_LLVM_CONFIG");
    println!("cargo:rerun-if-env-changed=SOPLANG_LLVM_LIB_DIR");
    if target_os == "windows" {
        let dir = env::var_os("SOPLANG_LLVM_LIB_DIR")
            .map(PathBuf::from)
            .expect("Windows embedded LLVM builds need SOPLANG_LLVM_LIB_DIR");
        println!("cargo:rustc-link-search=native={}", dir.display());
        println!("cargo:rustc-link-lib=dylib=LLVM-C");
        return;
    }
    let llvm_config = env::var_os("SOPLANG_LLVM_CONFIG").unwrap_or_else(|| "llvm-config".into());
    let libdir = Command::new(&llvm_config)
        .arg("--libdir")
        .output()
        .expect("embedded LLVM builds need llvm-config during compilation");
    assert!(
        libdir.status.success(),
        "llvm-config could not report its library directory"
    );
    println!(
        "cargo:rustc-link-search=native={}",
        String::from_utf8(libdir.stdout)
            .expect("invalid llvm-config library directory")
            .trim()
    );
    let output = Command::new(&llvm_config)
        .args([
            "--link-shared",
            "--libs",
            "core",
            "irreader",
            "passes",
            "nativecodegen",
        ])
        .output()
        .expect("embedded LLVM builds need llvm-config during compilation");
    assert!(
        output.status.success(),
        "llvm-config could not report LLVM libraries"
    );
    for flag in String::from_utf8(output.stdout)
        .expect("invalid llvm-config output")
        .split_whitespace()
    {
        if let Some(name) = flag.strip_prefix("-l") {
            println!("cargo:rustc-link-lib=dylib={name}");
        } else if let Some(path) = flag.strip_prefix("-L") {
            println!("cargo:rustc-link-search=native={path}");
        } else {
            panic!("unrecognized llvm-config library flag: {flag}");
        }
    }

    if target_os == "macos" {
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/lib/soplang/llvm");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../lib/soplang/llvm");
    }
    if target_os == "linux" {
        println!("cargo:rerun-if-env-changed=SOPLANG_LLD_SDK");
        println!("cargo:rerun-if-changed=src/backend/lld_bridge.cpp");
        let sdk = env::var_os("SOPLANG_LLD_SDK")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/usr"));
        let out = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo did not set OUT_DIR"));
        let bridge = out.join("lld_bridge.o");
        let archive = out.join("liblld_bridge.a");
        let status = Command::new("c++")
            .arg("-std=c++17")
            .arg("-fPIC")
            .arg("-I")
            .arg(sdk.join("include"))
            .arg("-c")
            .arg("src/backend/lld_bridge.cpp")
            .arg("-o")
            .arg(&bridge)
            .status()
            .expect("C++ compiler is needed to build the LLD bridge");
        assert!(status.success(), "LLD bridge compilation failed");
        let status = Command::new("ar")
            .arg("crs")
            .arg(&archive)
            .arg(&bridge)
            .status()
            .expect("archiver is needed to build the LLD bridge");
        assert!(status.success(), "LLD bridge archiving failed");
        println!("cargo:rustc-link-search=native={}", out.display());
        println!(
            "cargo:rustc-link-search=native={}",
            sdk.join("lib").display()
        );
        println!("cargo:rustc-link-lib=static=lld_bridge");
        let lld_kind = if sdk.join("lib/liblldELF.so").exists() {
            "dylib"
        } else if sdk.join("lib/liblldELF.a").exists() {
            "static"
        } else {
            panic!("LLD libraries were not found under {}", sdk.display());
        };
        println!("cargo:rustc-link-lib={lld_kind}=lldELF");
        println!("cargo:rustc-link-lib={lld_kind}=lldCommon");
        if lld_kind == "static" {
            println!("cargo:rustc-link-lib=dylib=zstd");
        }
        println!("cargo:rustc-link-lib=dylib=stdc++");
        println!("cargo:rustc-link-arg=-Wl,--disable-new-dtags");
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/lib/soplang/llvm");
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../lib/soplang/llvm");
    }
}
