// LVGL 9.6.0 bindgen + cc build (LVGL 9 line; originally ported from the
// mLupine-lvgl-9 branch which vendored 9.2.2).
//
// Vendored LVGL: vendor/lvgl-9.6/  (LVGL v9.6.0 release tarball
//   SHA256 b20ee3acc1bba13c62d854f9ebd62e4c51e0b443b1e0225892e86442defa84df
//   from https://github.com/lvgl/lvgl/archive/refs/tags/v9.6.0.tar.gz;
//   trimmed to the root build files + src/ + include/)
// Vendored config: vendor/include-9.6/lv_conf.h (derived from upstream
//   lv_conf_template.h with LV_USE_SNAPSHOT=1)
//
// On-target (ESP-IDF) consumers may instead link the LVGL shipped by
// `esp_lvgl_port` (managed component lvgl__lvgl); this vendored copy is the
// host-side cargo-check target and the source of truth for bindgen-generated
// FFI types. Both produce binary-compatible structs because both compile
// against `lv_conf.h` settings that match.

use cc::Build;
use std::{
    env,
    path::{Path, PathBuf},
};

fn main() {
    let project_dir = canonicalize(PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()));
    let shims_dir = project_dir.join("shims");
    let vendor = project_dir.join("vendor");
    let lvgl_root = vendor.join("lvgl-9.6");
    let lvgl_src = lvgl_root.join("src");
    let lv_config_dir = vendor.join("include-9.6");

    if !lv_config_dir.join("lv_conf.h").exists() {
        panic!(
            "missing {} -- vendor LVGL 9.6 lv_conf.h not staged",
            lv_config_dir.join("lv_conf.h").display()
        );
    }

    println!(
        "cargo:rerun-if-changed={}",
        lv_config_dir.join("lv_conf.h").to_str().unwrap()
    );
    println!("cargo:rerun-if-changed={}", shims_dir.to_str().unwrap());
    println!("cargo:rerun-if-changed={}", lvgl_src.to_str().unwrap());

    // SDL2 desktop-simulator backend (LV_USE_SDL), enabled by the `sdl`
    // feature. The vendored lv_conf.h keeps LV_USE_SDL=0 so default builds
    // (e.g. ESP cross-compiles) never need SDL headers; the feature build
    // overrides it through a generated LV_CONF_PATH header.
    let sdl_flags = if env::var("CARGO_FEATURE_SDL").is_ok() {
        let flags = discover_sdl2().expect(
            "feature `sdl` requires SDL2: install it (brew install sdl2) \
             so `pkg-config sdl2` or `sdl2-config` works",
        );
        let conf_override = PathBuf::from(env::var("OUT_DIR").unwrap()).join("lv_conf_sdl.h");
        std::fs::write(
            &conf_override,
            // Resolve the vendored config via the -I include-9.6 include path,
            // then flip the SDL backend on for this build only. Logging goes
            // to stdout so simulator runs can be diagnosed from the console.
            r#"#include "lv_conf.h"

#undef LV_USE_SDL
#define LV_USE_SDL 1

/* The embedded TLSF pool (LV_MEM_SIZE, 64 kB) is far too small to host a
 * desktop-size framebuffer snapshot; the simulator allocates from the C
 * heap instead. Embedded builds keep the built-in pool. */
#undef LV_USE_STDLIB_MALLOC
#define LV_USE_STDLIB_MALLOC LV_STDLIB_CLIB

#undef LV_USE_LOG
#define LV_USE_LOG 1
#undef LV_LOG_LEVEL
#define LV_LOG_LEVEL LV_LOG_LEVEL_WARN
#undef LV_LOG_PRINTF
#define LV_LOG_PRINTF 1
"#,
        )
        .expect("write lv_conf_sdl.h");        Some((flags, conf_override))
    } else {
        None
    };

    let mut cfg = Build::new();
    add_c_files(&mut cfg, &lvgl_src);
    add_c_files(&mut cfg, &shims_dir);

    cfg.define("LV_CONF_INCLUDE_SIMPLE", Some("1"))
        .include(&lvgl_root)
        .include(&lvgl_src)
        .include(&lv_config_dir)
        .include(&vendor)
        .warnings(false);

    if let Some((sdl, conf_override)) = &sdl_flags {
        // LV_CONF_PATH wins over LV_CONF_INCLUDE_SIMPLE inside
        // lv_conf_internal.h and must be a C string literal.
        let conf_define = format!("{:?}", conf_override.display());
        cfg.define("LV_CONF_PATH", Some(conf_define.as_str()));
        for i in &sdl.include_dirs {
            cfg.include(i);
        }
        for (k, v) in &sdl.defines {
            cfg.define(k, Some(v.as_str()));
        }
    }

    let cflags_extra = env::var("LVGL_CFLAGS").unwrap_or_default();
    let cflags_extra: Vec<&str> = cflags_extra.split(',').filter(|s| !s.is_empty()).collect();
    for e in &cflags_extra {
        let mut it = e.split('=');
        cfg.define(it.next().unwrap(), it.next().unwrap_or_default());
    }

    cfg.compile("lvgl");

    if let Some((sdl, _)) = &sdl_flags {
        for dir in &sdl.link_dirs {
            println!("cargo:rustc-link-search=native={dir}");
        }
        for lib in &sdl.libs {
            println!("cargo:rustc-link-lib=dylib={lib}");
        }
        for fw in &sdl.frameworks {
            println!("cargo:rustc-link-lib=framework={fw}");
        }
    }

    let mut cc_args = vec![
        "-DLV_CONF_INCLUDE_SIMPLE=1".to_string(),
        format!("-I{}", lv_config_dir.display()),
        format!("-I{}", lvgl_root.display()),
        format!("-I{}", lvgl_src.display()),
        format!("-I{}", vendor.display()),
        "-fvisibility=default".to_string(),
    ];
    for e in &cflags_extra {
        cc_args.push(format!("-D{e}"));
    }

    let target = env::var("TARGET").expect("Cargo build scripts always have TARGET");
    let host = env::var("HOST").expect("Cargo build scripts always have HOST");
    if target != host {
        cc_args.push("-target".to_string());
        cc_args.push(target.clone());
    }

    // Bindgen invokes clang with `-target xtensa-esp32s3-espidf` and that
    // clang has no idea where the xtensa newlib headers live. Discover the
    // sysroot via xtensa-esp-elf-gcc on PATH and pass `-I <sysroot>/include`
    // so `<inttypes.h>` etc resolve. The same trick works for any
    // `xtensa-*-espidf` target. If the toolchain isn't on PATH (host build)
    // we silently skip -- there's nothing to add.
    if target.starts_with("xtensa-") {
        if let Some(sysroot_inc) = xtensa_sysroot_include() {
            cc_args.push(format!("-I{}", sysroot_inc));
        }
    }

    if let Some((sdl, conf_override)) = &sdl_flags {
        cc_args.push(format!("-DLV_CONF_PATH={:?}", conf_override.display()));
        for i in &sdl.include_dirs {
            cc_args.push(format!("-I{}", i));
        }
        for (k, v) in &sdl.defines {
            if v.is_empty() {
                cc_args.push(format!("-D{k}"));
            } else {
                cc_args.push(format!("-D{k}={v}"));
            }
        }
    }

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bindings = bindgen::Builder::default()
        .header(shims_dir.join("lvgl_sys.h").to_str().unwrap())
        .generate_comments(false)
        .derive_default(true)
        .layout_tests(false)
        .use_core()
        .ctypes_prefix("cty")
        .clang_args(cc_args.iter().map(|s| s.as_str()))
        .allowlist_type("lv_.*")
        .allowlist_type("_lv_.*")
        .allowlist_function("lv_.*")
        .allowlist_function("_lv_.*")
        // The custom color-channel accessors live in our shim
        // (lvgl-sys/shims/lvgl_sys.{h,c}) and are spelled `_LV_COLOR_*`.
        .allowlist_function("_LV_COLOR_.*")
        .allowlist_var("LV_.*")
        // Built-in fonts are extern `lv_font_t` variables (lowercase), e.g.
        // lv_font_montserrat_14; expose the ones enabled in lv_conf.h.
        .allowlist_var("lv_font_.*")
        .blocklist_function("lv_log_add") // varargs (va_list); not safely bindable on all targets
        .generate()
        .expect("Unable to generate LVGL 9.6 bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Can't write bindings!");
}

fn add_c_files(build: &mut cc::Build, path: impl AsRef<Path>) {
    for e in path.as_ref().read_dir().expect("read_dir failed") {
        let e = e.unwrap();
        let p = e.path();
        if e.file_type().unwrap().is_dir() {
            add_c_files(build, &p);
        } else if p.extension().and_then(|s| s.to_str()) == Some("c") {
            build.file(&p);
        }
    }
}

/// Compile/link flags for SDL2, discovered via `pkg-config sdl2` with a
/// `sdl2-config` fallback (Homebrew ships both).
struct SdlFlags {
    include_dirs: Vec<String>,
    defines: Vec<(String, String)>,
    link_dirs: Vec<String>,
    libs: Vec<String>,
    frameworks: Vec<String>,
}

fn discover_sdl2() -> Option<SdlFlags> {
    use std::process::Command;

    let mut flags = SdlFlags {
        include_dirs: Vec::new(),
        defines: Vec::new(),
        link_dirs: Vec::new(),
        libs: Vec::new(),
        frameworks: Vec::new(),
    };
    let mut saw_any = false;

    let mut absorb = |out: &str| {
        let mut fw_next = false;
        for tok in out.split_whitespace() {
            if fw_next {
                flags.frameworks.push(tok.to_string());
                fw_next = false;
            } else if let Some(dir) = tok.strip_prefix("-I") {
                if !dir.is_empty() {
                    flags.include_dirs.push(dir.to_string());
                }
            } else if let Some(def) = tok.strip_prefix("-D") {
                let (k, v) = def.split_once('=').unwrap_or((def, ""));
                flags.defines.push((k.to_string(), v.to_string()));
            } else if let Some(dir) = tok.strip_prefix("-L") {
                if !dir.is_empty() {
                    flags.link_dirs.push(dir.to_string());
                }
            } else if let Some(lib) = tok.strip_prefix("-l") {
                if !lib.is_empty() {
                    flags.libs.push(lib.to_string());
                }
            } else if tok == "-framework" {
                fw_next = true;
            }
        }
    };

    if let Ok(out) = Command::new("pkg-config").args(["--cflags", "sdl2"]).output() {
        if out.status.success() {
            absorb(&String::from_utf8_lossy(&out.stdout));
            saw_any = true;
        }
    }
    if let Ok(out) = Command::new("pkg-config").args(["--libs", "sdl2"]).output() {
        if out.status.success() {
            absorb(&String::from_utf8_lossy(&out.stdout));
            saw_any = true;
        }
    }
    if !saw_any {
        if let Ok(out) = Command::new("sdl2-config").arg("--cflags").output() {
            if out.status.success() {
                absorb(&String::from_utf8_lossy(&out.stdout));
                saw_any = true;
            }
        }
        if let Ok(out) = Command::new("sdl2-config").arg("--libs").output() {
            if out.status.success() {
                absorb(&String::from_utf8_lossy(&out.stdout));
                saw_any = true;
            }
        }
    }
    if !saw_any {
        return None;
    }
    Some(flags)
}

/// Locate the xtensa-esp-elf newlib include dir by asking
/// `xtensa-esp-elf-gcc -print-sysroot` (then appending `/include`).
/// Returns `None` if the tool is missing or fails.
fn xtensa_sysroot_include() -> Option<String> {
    use std::process::Command;
    let out = Command::new("xtensa-esp-elf-gcc")
        .arg("-print-sysroot")
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let sysroot = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if sysroot.is_empty() {
        return None;
    }
    Some(format!("{sysroot}/include"))
}

fn canonicalize(path: impl AsRef<Path>) -> PathBuf {
    let canonicalized = path.as_ref().canonicalize().unwrap();
    let s = canonicalized.to_string_lossy().to_string();
    PathBuf::from(s.strip_prefix(r"\\?\").unwrap_or(s.as_str()))
}
