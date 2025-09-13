#[cfg(feature = "rebuild-bindings")]
extern crate bindgen;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::env;
    use std::fs::{self};
    use std::path::{Path, PathBuf};

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let sdk_loc = if let Ok(sdk_loc) = env::var("STEAM_SDK_LOCATION") {
        Path::new(&sdk_loc).to_path_buf()
    } else {
        let mut path = PathBuf::new();
        path.push(env::var("CARGO_MANIFEST_DIR").unwrap());
        path.push("lib");
        path.push("steam");
        path
    };
    println!("cargo:rerun-if-env-changed=STEAM_SDK_LOCATION");

    let triple = env::var("TARGET").unwrap();
    let mut lib = "steam_api";
    let mut link_path = sdk_loc.join("redistributable_bin");
    if triple.contains("windows") {
        if !triple.contains("i686") {
            lib = "steam_api64";
            link_path.push("win64");
        }
    } else if triple.contains("linux") {
        if triple.contains("i686") {
            link_path.push("linux32");
        } else {
            link_path.push("linux64");
        }
    } else if triple.contains("darwin") {
        link_path.push("osx");
    } else {
        panic!("Unsupported OS");
    };

    if triple.contains("windows") {
        let dll_file = format!("{}.dll", lib);
        let lib_file = format!("{}.lib", lib);
        fs::copy(link_path.join(&dll_file), out_path.join(dll_file))?;
        fs::copy(link_path.join(&lib_file), out_path.join(lib_file))?;
    } else if triple.contains("darwin") {
        fs::copy(
            link_path.join("libsteam_api.dylib"),
            out_path.join("libsteam_api.dylib"),
        )?;
    } else if triple.contains("linux") {
        fs::copy(
            link_path.join("libsteam_api.so"),
            out_path.join("libsteam_api.so"),
        )?;
    }

    println!("cargo:rustc-link-search={}", out_path.display());
    println!("cargo:rustc-link-lib=dylib={}", lib);

    // This is to avoid modifying the Steamworks SDK files directly.

    // Compile a small C++ wrapper that exposes ISteamGameCoordinator methods as C functions
    // This is required at link time because bindgen produces extern declarations for
    // SteamGC_* functions. We compile this wrapper unconditionally so examples and
    // downstream crates can link even when the `rebuild-bindings` feature is not set.
    // Expect checked-in wrapper sources to exist. This avoids mutating the repo at
    // build time and makes the project more explicit about source files.
    let wrapper_cpp = Path::new("lib").join("steam_gc_wrapper.cpp");
    let wrapper_h = Path::new("lib").join("steam_gc_wrapper.h");
    if !wrapper_cpp.exists() {
        return Err(format!(
            "Missing required file: {}. Please ensure lib/steam_gc_wrapper.cpp is present.",
            wrapper_cpp.display()
        )
        .into());
    }
    if !wrapper_h.exists() {
        return Err(format!(
            "Missing required file: {}. Please ensure lib/steam_gc_wrapper.h is present.",
            wrapper_h.display()
        )
        .into());
    }

    // compile the wrapper into OUT_DIR so the linker can find it for tests and examples
    cc::Build::new()
        .cpp(true)
        .file(&wrapper_cpp)
        .include(sdk_loc.join("public"))
        .compile("steam_gc_wrapper");

    // Ensure the compiled wrapper library is linked into downstream crates. The `cc`
    // crate will produce a static library named libsteam_gc_wrapper.a on unix-like
    // systems and steam_gc_wrapper.lib on MSVC. Emit the appropriate cargo directive
    // so the final link step pulls it in.
    println!("cargo:rustc-link-lib=static=steam_gc_wrapper");

    #[cfg(feature = "rebuild-bindings")]
    {
        let target_os = if triple.contains("windows") {
            "windows"
        } else if triple.contains("darwin") {
            "macos"
        } else if triple.contains("linux") {
            "linux"
        } else {
            panic!("Unsupported OS");
        };

        let binding_path = Path::new(&format!("src/{}_bindings.rs", target_os)).to_owned();
        let bindings = bindgen::Builder::default()
            .header(
                sdk_loc
                    .join("public/steam/steam_api_flat.h")
                    .to_string_lossy(),
            )
            // ensure wrapper header is parsed so the C functions are generated
            .header(wrapper_h.to_string_lossy())
            .header(
                sdk_loc
                    .join("public/steam/steam_gameserver.h")
                    .to_string_lossy(),
            )
            .header(
                sdk_loc
                    .join("public/steam/isteamgamecoordinator.h")
                    .to_string_lossy(),
            )
            .clang_arg("-xc++")
            .clang_arg("-std=c++11")
            .clang_arg(format!("-I{}", sdk_loc.join("public").display()))
            .allowlist_function("Steam.*")
            .allowlist_var(".*") // TODO: Prune constants
            .allowlist_type(".*") // TODO: Prune types
            .default_enum_style(bindgen::EnumVariation::Rust {
                non_exhaustive: true,
            })
            .bitfield_enum("EMarketNotAllowedReasonFlags")
            .bitfield_enum("EBetaBranchFlags")
            .bitfield_enum("EFriendFlags")
            .bitfield_enum("EPersonaChange")
            .bitfield_enum("ERemoteStoragePlatform")
            .bitfield_enum("EChatSteamIDInstanceFlags")
            .bitfield_enum("ESteamItemFlags")
            .bitfield_enum("EOverlayToStoreFlag")
            .bitfield_enum("EChatSteamIDInstanceFlags")
            .generate()
            .expect("Unable to generate bindings");

        bindings
            .write_to_file(binding_path)
            .expect("Couldn't write bindings!");
    }

    Ok(())
}
