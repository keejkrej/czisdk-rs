use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=LIBCZI_INCLUDE_DIR");
    println!("cargo:rerun-if-env-changed=LIBCZI_LIB_DIR");
    println!("cargo:rerun-if-env-changed=LIBCZI_LIB_NAME");
    println!("cargo:rerun-if-env-changed=LIBCZI_STATIC");
    println!("cargo:rerun-if-env-changed=VCPKG_ROOT");
    println!("cargo:rerun-if-env-changed=VCPKGRS_TRIPLET");
    println!("cargo:rerun-if-changed=native/czisdk_rs.cpp");

    let mut include_dirs = Vec::new();
    let static_link;

    if let (Ok(include_dir), Ok(lib_dir)) =
        (env::var("LIBCZI_INCLUDE_DIR"), env::var("LIBCZI_LIB_DIR"))
    {
        include_dirs.push(PathBuf::from(include_dir));
        println!("cargo:rustc-link-search=native={lib_dir}");
        let kind = if env::var_os("LIBCZI_STATIC").is_some() {
            "static"
        } else {
            "dylib"
        };
        static_link = kind == "static";
        let lib_name =
            env::var("LIBCZI_LIB_NAME").unwrap_or_else(|_| default_lib_name(static_link));
        println!("cargo:rustc-link-lib={kind}={lib_name}");
    } else if let Ok(library) = vcpkg::Config::new()
        .cargo_metadata(false)
        .emit_includes(true)
        .find_package("libczi")
    {
        static_link = library.is_static;
        let staged_lib_dir = stage_link_libraries(
            &PathBuf::from(env::var_os("OUT_DIR").unwrap_or_else(|| Path::new(".").into())),
            &library.found_libs,
        );
        println!(
            "cargo:rustc-link-search=native={}",
            staged_lib_dir.display()
        );
        for found_lib in &library.found_libs {
            let link_name = link_name_for_path(found_lib);
            let kind = if library.is_static { "static" } else { "dylib" };
            println!("cargo:rustc-link-lib={kind}={link_name}");
        }
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

    if static_link {
        build.define("_LIBCZISTATICLIB", None);

        if cfg!(target_os = "windows") {
            println!("cargo:rustc-link-lib=dylib=windowscodecs");
        }
    }

    for include_dir in include_dirs {
        build.include(&include_dir);
        build.include(include_dir.join("libCZI"));
    }

    build.compile("czisdk_rs_bridge");
}

fn stage_link_libraries(out_dir: &Path, libs: &[PathBuf]) -> PathBuf {
    let staged_dir = out_dir.join("czi-rs-links");
    fs::create_dir_all(&staged_dir).expect("unable to create staged czi library directory");

    for lib in libs {
        let lib_name = lib.file_name().unwrap_or_else(|| lib.as_os_str());
        let staged_lib = staged_dir.join(lib_name);
        if !staged_lib.exists() {
            fs::copy(lib, &staged_lib)
                .unwrap_or_else(|err| panic!("failed to stage czi library {}: {err}", lib.display()));
        }
    }

    staged_dir
}

fn link_name_for_path(path: &Path) -> String {
    let stem = path
        .file_stem()
        .expect("vcpkg library path has no file stem")
        .to_string_lossy();

    if path.extension().and_then(|extension| extension.to_str()) == Some("a") {
        stem.strip_prefix("lib").unwrap_or(&stem).to_owned()
    } else {
        stem.into_owned()
    }
}

fn default_lib_name(static_link: bool) -> String {
    if cfg!(target_os = "windows") && static_link {
        "libCZIStatic".to_owned()
    } else if cfg!(target_os = "windows") {
        "libCZI".to_owned()
    } else {
        "CZI".to_owned()
    }
}
