use std::path::PathBuf;

macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

fn main() -> miette::Result<()> {
    tauri_build::build();

    let path = PathBuf::from("src");
    let include_path = PathBuf::from("C:\\SLDevice\\SDK\\headers");
    let mut b = autocxx_build::Builder::new("src/wrapper.rs", [&path, &include_path]).build()?;
    b.flag_if_supported("-std=c++14").compile("autocxx-demo");

    let lib_path = "C:\\SLDevice\\SDK\\lib\\x64\\Release";
    println!("cargo:rustc-link-search={}", lib_path);
    println!("cargo:rustc-link-lib=SLImage");
    println!("cargo:rustc-link-lib=SLDeviceLib");
    println!("cargo:rustc-link-lib=libtiff");
    Ok(())
}
