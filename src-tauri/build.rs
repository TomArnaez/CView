use std::path::PathBuf;

macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

fn main() -> miette::Result<()> {
    let mut windows = tauri_build::WindowsAttributes::new();
    windows = windows.app_manifest(r#"
    <assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
    <dependency>
        <dependentAssembly>
        <assemblyIdentity
            type="win32"
            name="Microsoft.Windows.Common-Controls"
            version="6.0.0.0"
            processorArchitecture="*"
            publicKeyToken="6595b64144ccf1df"
            language="*"
        />
        </dependentAssembly>
    </dependency>
    <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
        <security>
            <requestedPrivileges>
                <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
            </requestedPrivileges>
        </security>
    </trustInfo>
    </assembly>
    "#);

    tauri_build::try_build(  tauri_build::Attributes::new().windows_attributes(windows));

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
