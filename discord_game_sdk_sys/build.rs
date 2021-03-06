use std::{env, error::Error, path::PathBuf};

// TODO make sure references are used when they should be
#[cfg(feature = "download")]
fn dl_sdk(sdkfolder: PathBuf) -> Result<PathBuf, Box<dyn Error>> {
    use std::fs::{self, File};
    use std::io::{self, Cursor};

    use reqwest::blocking::get;
    use zip::read::ZipArchive;

    // Download and extract SDK
    let req = get("https://dl-game-sdk.discordapp.net/latest/discord_game_sdk.zip")?
        .error_for_status()?;
    let mut req = Cursor::new(req.bytes()?);
    let mut archive = ZipArchive::new(&mut req)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = sdkfolder.join(file.sanitized_name());

        if (&*file.name()).ends_with('/') {
            // Folder
            if !outpath.exists() {
                fs::create_dir_all(&outpath)?;
            }
        } else {
            // File
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }

    println!("SDK extracted to: \"{}\"", sdkfolder.display());

    rn_sdk(sdkfolder.join("lib/"))?;
    Ok(sdkfolder)
}

#[cfg(not(feature = "download"))]
fn dl_sdk(_: PathBuf) -> Result<PathBuf, Box<dyn Error>> {
    Err(std::io::Error::new(std::io::ErrorKind::Other, "Download disabled").into())
}

#[cfg(feature = "download")]
fn rn_sdk(libfolder: PathBuf) -> Result<(), Box<dyn Error>> {
    use std::fs;

    for x in &["x86/", "x86_64/"] {
        for entry in fs::read_dir(libfolder.join(x))? {
            let entry = entry?;
            let path = entry.path();
            let ext = path
                .extension()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default();
            let file_name = path
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default();

            if !file_name.starts_with("lib") {
                match ext {
                    "so" | "dylib" => {
                        fs::rename(&path, path.with_file_name("lib".to_owned() + file_name))?;
                    }
                    _ => (),
                }
            }

            if file_name.ends_with(".dll.lib") {
                fs::rename(
                    &path,
                    path.with_file_name((&file_name[..file_name.len() - 8]).to_owned() + ".lib"),
                )?;
            }
        }
    }

    Ok(())
}

fn main() {
    let target = env::var("TARGET").unwrap();
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    // DO NOT RELY ON THIS
    if cfg!(feature = "doc") {
        std::fs::copy("src/.generated.rs", out_path.join("bindings.rs")).unwrap();
        return;
    }

    let sdk_path = dl_sdk(out_path.join("sdk/")).unwrap_or_else(|_| {
        PathBuf::from(env::var("DISCORD_GAME_SDK_PATH").expect(MISSING_SDK_PATH))
    });

    println!("cargo:rerun-if-env-changed=DISCORD_GAME_SDK_PATH");
    println!("cargo:rerun-if-changed={}", sdk_path.to_str().unwrap());

    bindgen::builder()
        .header(sdk_path.join("c/discord_game_sdk.h").to_str().unwrap())
        .ctypes_prefix("ctypes")
        .derive_copy(true)
        .derive_debug(true)
        .derive_default(true)
        .derive_eq(true)
        .derive_hash(true)
        .derive_partialeq(true)
        .generate_comments(false)
        .impl_debug(true)
        .impl_partialeq(true)
        .parse_callbacks(Box::new(Callbacks))
        .prepend_enum_name(false)
        .whitelist_function("Discord.+")
        .whitelist_type("[EI]?Discord.+")
        .whitelist_var("DISCORD_.+")
        .generate()
        .expect("discord_game_sdk_sys: bindgen could not generate bindings")
        .write_to_file(out_path.join("bindings.rs"))
        .expect("discord_game_sdk_sys: could not write bindings to file");

    std::fs::copy(out_path.join("bindings.rs"), "src/.generated.rs").unwrap();

    if cfg!(not(feature = "link")) {
        return;
    }

    match target.as_ref() {
        "x86_64-unknown-linux-gnu" => {
            assert!(
                sdk_path.join("lib/x86_64/libdiscord_game_sdk.so").exists(),
                MISSING_SETUP
            );
        }

        "x86_64-apple-darwin" => {
            // TODO: assert SDK is in DYLD_LIBRARY_PATH
            assert!(
                sdk_path
                    .join("lib/x86_64/libdiscord_game_sdk.dylib")
                    .exists(),
                MISSING_SETUP
            );
        }

        "x86_64-pc-windows-gnu" | "x86_64-pc-windows-msvc" => {
            assert!(
                sdk_path.join("lib/x86_64/discord_game_sdk.lib").exists(),
                MISSING_SETUP
            );
        }

        "i686-pc-windows-gnu" | "i686-pc-windows-msvc" => {
            assert!(
                sdk_path.join("lib/x86/discord_game_sdk.lib").exists(),
                MISSING_SETUP
            );
        }

        _ => panic!(INCOMPATIBLE_PLATFORM),
    }

    match target.as_ref() {
        "x86_64-unknown-linux-gnu"
        | "x86_64-apple-darwin"
        | "x86_64-pc-windows-gnu"
        | "x86_64-pc-windows-msvc" => {
            println!("cargo:rustc-link-lib=discord_game_sdk");
            println!(
                "cargo:rustc-link-search={}",
                sdk_path.join("lib/x86_64").to_str().unwrap()
            );
        }

        "i686-pc-windows-gnu" | "i686-pc-windows-msvc" => {
            println!("cargo:rustc-link-lib=discord_game_sdk");
            println!(
                "cargo:rustc-link-search={}",
                sdk_path.join("lib/x86").to_str().unwrap()
            );
        }

        _ => {}
    }
}

#[derive(Debug)]
struct Callbacks;

impl bindgen::callbacks::ParseCallbacks for Callbacks {
    fn int_macro(&self, name: &str, _value: i64) -> Option<bindgen::callbacks::IntKind> {
        // Must match sys::DiscordVersion
        if name.ends_with("_VERSION") {
            Some(bindgen::callbacks::IntKind::I32)
        } else {
            None
        }
    }
}

const MISSING_SDK_PATH: &str = r#"

discord_game_sdk_sys: Hello,

You are trying to generate the bindings for the Discord Game SDK.
You will have to download the SDK yourself.
Here are the links to get it:

https://discordapp.com/developers/docs/game-sdk/sdk-starter-guide
https://dl-game-sdk.discordapp.net/latest/discord_game_sdk.zip

Once you have downloaded it, extract the contents to a folder
and set the environment variable `DISCORD_GAME_SDK_PATH` to its path.

Example:

$ export DISCORD_GAME_SDK_PATH=$HOME/Downloads/discord_game_sdk

Please report any issues you have at:
https://github.com/ldesgoui/discord_game_sdk

Thanks, and apologies for the inconvenience

"#;

const MISSING_SETUP: &str = r#"

discord_game_sdk_sys: Hello,

You are trying to link to the Discord Game SDK.
Some additional set-up is required, namely some files need to be copied for the linker:

# Linux: prepend with `lib` and add to library search path
$ cp $DISCORD_GAME_SDK_PATH/lib/x86_64/{,lib}discord_game_sdk.so
$ export LD_LIBRARY_PATH=${LD_LIBRARY_PATH:+${LD_LIBRARY_PATH}:}$DISCORD_GAME_SDK_PATH/lib/x86_64

# Mac OS: prepend with `lib` and add to library search path
$ cp $DISCORD_GAME_SDK_PATH/lib/x86_64/{,lib}discord_game_sdk.dylib
$ export DYLD_LIBRARY_PATH=${DYLD_LIBRARY_PATH:+${DYLD_LIBRARY_PATH}:}$DISCORD_GAME_SDK_PATH/lib/x86_64

# Windows: copy `*.dll.lib` to `*.lib` (won't affect library search)
$ cp $DISCORD_GAME_SDK_PATH/lib/x86_64/discord_game_sdk.{dll.lib,lib}
$ cp $DISCORD_GAME_SDK_PATH/lib/x86/discord_game_sdk.{dll.lib,lib}

After all this, `cargo build` and `cargo run` should function as expected.

Please report any issues you have at:
https://github.com/ldesgoui/discord_game_sdk

Thanks, and apologies for the inconvenience

"#;

const INCOMPATIBLE_PLATFORM: &str = r#"

discord_game_sdk_sys: Hello,

You are trying to link to the Discord Game SDK.
Unfortunately, the platform you are trying to target is not supported.

Please report any issues you have at:
https://github.com/ldesgoui/discord_game_sdk

Thanks, and apologies for the inconvenience

"#;
