// LVGL 9.2.2 bindgen + cc build for the mLupine fork (mLupine-lvgl-9 branch).
//
// Vendored LVGL: vendor/lvgl-9.2/  (LVGL v9.2.2 release tarball
//   SHA256 129b4e00e06639fa79d7e8a6cab3c1ecce2445b1a246652ccd34f22e7b17ad6f
//   from https://github.com/lvgl/lvgl/archive/refs/tags/v9.2.2.tar.gz)
// Vendored config: vendor/include-9.2/lv_conf.h (derived from upstream
//   lv_conf_template.h with LV_USE_SNAPSHOT=1)
//
// On-target (ESP-IDF v6) consumers of `lupin-display` will use the LVGL
// shipped by `esp_lvgl_port` (managed component lvgl__lvgl ~9.2.x); this
// vendored copy is the host-side cargo-check target and the source of truth
// for bindgen-generated FFI types. Both produce binary-compatible structs
// because both compile against `lv_conf.h` settings that match.

use cc::Build;
use std::{
    env,
    path::{Path, PathBuf},
};

fn main() {
    let project_dir = canonicalize(PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()));
    let shims_dir = project_dir.join("shims");
    let vendor = project_dir.join("vendor");
    let lvgl_root = vendor.join("lvgl-9.2");
    let lvgl_src = lvgl_root.join("src");
    let lv_config_dir = vendor.join("include-9.2");

    if !lv_config_dir.join("lv_conf.h").exists() {
        panic!(
            "missing {} -- vendor LVGL 9.2 lv_conf.h not staged",
            lv_config_dir.join("lv_conf.h").display()
        );
    }

    println!(
        "cargo:rerun-if-changed={}",
        lv_config_dir.join("lv_conf.h").to_str().unwrap()
    );
    println!("cargo:rerun-if-changed={}", shims_dir.to_str().unwrap());
    println!("cargo:rerun-if-changed={}", lvgl_src.to_str().unwrap());

    let mut cfg = Build::new();
    add_c_files(&mut cfg, &lvgl_src);
    add_c_files(&mut cfg, &shims_dir);

    cfg.define("LV_CONF_INCLUDE_SIMPLE", Some("1"))
        .include(&lvgl_root)
        .include(&lvgl_src)
        .include(&lv_config_dir)
        .include(&vendor)
        .warnings(false);

    let cflags_extra = env::var("LVGL_CFLAGS").unwrap_or_default();
    let cflags_extra: Vec<&str> = cflags_extra.split(',').filter(|s| !s.is_empty()).collect();
    for e in &cflags_extra {
        let mut it = e.split('=');
        cfg.define(it.next().unwrap(), it.next().unwrap_or_default());
    }

    cfg.compile("lvgl");

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
        .blocklist_function("lv_log_add") // varargs (va_list); not safely bindable on all targets
        .generate()
        .expect("Unable to generate LVGL 9.2 bindings");

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
