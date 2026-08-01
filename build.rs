use std::{
    ffi::OsStr,
    os::unix::process::ExitStatusExt,
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

fn compile_slang(dir: &Path, path: &PathBuf) {
    let name = path.file_prefix().unwrap();
    let mut output = String::from(name.to_str().unwrap());
    output.push_str(".spirv");
    output.insert_str(0, dir.to_str().unwrap());
    let _ = Command::new("slangc")
        .arg(path)
        .arg("-target")
        .arg("spirv")
        .arg("-o")
        .arg(output)
        .arg("-entry")
        .arg("main")
        .status()
        .unwrap();
}

fn main() {
    println!("cargo::rerun-if-changed=src/gpu/shaders");
    let mut out = PathBuf::from_str(&std::env::var("OUT_DIR").unwrap()).unwrap();
    out.push("shaders/");
    let _ = std::fs::create_dir(&out);
    println!("cargo::rustc-env=SHADER_OUT={}", out.display());
    let dirs = std::fs::read_dir("src/gpu/shaders/").unwrap();
    for item in dirs {
        let path = item.unwrap().path();

        if path.to_str().unwrap().contains("comp") {
            compile_slang(&out, &path);
        }
    }
}
