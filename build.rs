use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-env-changed=LIBCZI_INCLUDE_DIR");
    println!("cargo:rerun-if-env-changed=LIBCZI_LIB_DIR");
    println!("cargo:rerun-if-env-changed=LIBCZI_LIB_NAME");
    println!("cargo:rerun-if-env-changed=LIBCZI_STATIC");
    println!("cargo:rerun-if-env-changed=VCPKG_ROOT");
    println!("cargo:rerun-if-changed=native/czisdk_rs.cpp");

    let mut include_dirs = Vec::new();

    if let (Ok(include_dir), Ok(lib_dir)) =
        (env::var("LIBCZI_INCLUDE_DIR"), env::var("LIBCZI_LIB_DIR"))
    {
        include_dirs.push(PathBuf::from(include_dir));
        println!("cargo:rustc-link-search=native={lib_dir}");
        let lib_name = env::var("LIBCZI_LIB_NAME").unwrap_or_else(|_| "libCZI".to_owned());
        let kind = if env::var_os("LIBCZI_STATIC").is_some() {
            "static"
        } else {
            "dylib"
        };
        println!("cargo:rustc-link-lib={kind}={lib_name}");
    } else if let Ok(library) = vcpkg::Config::new()
        .emit_includes(true)
        .find_package("libczi")
    {
        include_dirs.extend(library.include_paths);
    } else {
        panic!(
            "libCZI was not found. Install vcpkg package 'libczi' or set \
             LIBCZI_INCLUDE_DIR and LIBCZI_LIB_DIR to an installed libCZI package."
        );
    }

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .std("c++17")
        .file("native/czisdk_rs.cpp")
        .warnings(false);

    for include_dir in include_dirs {
        build.include(&include_dir);
        build.include(include_dir.join("libCZI"));
    }

    build.compile("czisdk_rs_bridge");
}
