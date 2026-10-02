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
    // Public headers: bindgen resolves lvgl.h -> include/lvgl/**, so edits
    // there must retrigger both cc and bindgen.
    println!(
        "cargo:rerun-if-changed={}",
        lvgl_root.join("lvgl.h").to_str().unwrap()
    );
    println!(
        "cargo:rerun-if-changed={}",
        lvgl_root.join("lvgl_private.h").to_str().unwrap()
    );
    println!(
        "cargo:rerun-if-changed={}",
        lvgl_root.join("include").to_str().unwrap()
    );
    println!("cargo:rerun-if-env-changed=LVGL_CFLAGS");
    println!("cargo:rerun-if-env-changed=LVGL_SYSROOT");

    // SDL2 desktop-simulator backend (LV_USE_SDL), enabled by the `sdl`
    // feature. The vendored lv_conf.h keeps the backend off so default builds
    // (e.g. ESP cross-compiles) never need SDL headers; the feature build
    // prepends a patched copy of the config via include-path order.
    let sdl_flags = if env::var("CARGO_FEATURE_SDL").is_ok() {
        let flags = discover_sdl2().unwrap_or_else(|| {
            panic!(
                "feature `sdl` requires SDL2: install it (e.g. `brew install sdl2` \
                 or `apt install libsdl2-dev`) so `pkg-config sdl2` or `sdl2-config` works"
            )
        });
        let sdl_conf_dir = PathBuf::from(env::var("OUT_DIR").unwrap()).join("sdl-conf");
        std::fs::create_dir_all(&sdl_conf_dir).expect("create sdl-conf dir");
        let base_conf = std::fs::read_to_string(lv_config_dir.join("lv_conf.h"))
            .expect("read vendored lv_conf.h");
        std::fs::write(
            sdl_conf_dir.join("lv_conf.h"),
            patch_lv_conf_for_sdl(&base_conf),
        )
        .expect("write sdl-conf/lv_conf.h");
        Some((flags, sdl_conf_dir))
    } else {
        None
    };

    let mut cfg = Build::new();
    add_c_files(&mut cfg, &lvgl_src);
    add_c_files(&mut cfg, &shims_dir);

    // Official widgets demo (`demo-widgets` cargo feature): compile the
    // vendored demo sources. LV_USE_DEMO_WIDGETS itself is enabled in the
    // vendored lv_conf.h.
    let demo_widgets = env::var("CARGO_FEATURE_LV_DEMO_WIDGETS").is_ok();
    if demo_widgets {
        let widgets_dir = lvgl_root.join("demos").join("widgets");
        add_c_files(&mut cfg, &widgets_dir);
        // lv_demo_widgets refers to the shared demo-args helper, which is
        // defined at demos/lv_demos.c (other demos' calls compile out).
        cfg.file(lvgl_root.join("demos").join("lv_demos.c"));
    }

    cfg.define("LV_CONF_INCLUDE_SIMPLE", Some("1"));
    if let Some((_, sdl_conf_dir)) = &sdl_flags {
        // Must precede lv_config_dir: quoted includes search -I in order, so
        // `#include "lv_conf.h"` resolves to the patched copy, not the
        // vendored one.
        cfg.include(sdl_conf_dir);
    }
    cfg.include(&lvgl_root)
        .include(&lvgl_src)
        .include(&lv_config_dir)
        .include(&vendor)
        .warnings(false);

    if let Some((sdl, _)) = &sdl_flags {
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
    let triple;
    let mut arch_args: Vec<String> = Vec::new();
    if let Some(isa) = target
        .strip_prefix("riscv32")
        .and_then(|rest| rest.split('-').next())
    {
        // Rust-style triples ("riscv32imafc-unknown-none-elf") are not valid
        // clang *driver* triples; pass the canonical LLVM triple plus explicit
        // ISA/ABI (single-arg -march=value form: libclang rejects the split
        // argv form) so bindgen computes layout identically to the C compiler.
        triple = "riscv32-unknown-elf".to_string();
        let mabi = if isa.contains('d') {
            "ilp32d"
        } else if isa.contains('f') {
            "ilp32f"
        } else {
            "ilp32"
        };
        arch_args.push(format!("-march=rv32{isa}"));
        arch_args.push(format!("-mabi={mabi}"));
    } else {
        triple = target.clone();
    }
    if target != host {
        cc_args.push("-target".to_string());
        cc_args.push(triple);
        cc_args.extend(arch_args);
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

    if let Some((sdl, sdl_conf_dir)) = &sdl_flags {
        // Insert before lv_config_dir (cc_args[1]) so `#include "lv_conf.h"`
        // resolves to the patched copy, mirroring the cc include order.
        cc_args.insert(1, format!("-I{}", sdl_conf_dir.display()));
        for i in &sdl.include_dirs {
            cc_args.push(format!("-I{}", i));
        }
        for (k, v) in &sdl.defines {
            cc_args.push(format!("-D{k}={v}"));
        }
    }

    if demo_widgets {
        cc_args.push(format!(
            "-I{}",
            lvgl_root.join("demos").join("widgets").display()
        ));
    }

    // bindgen runs against the host libclang, which has no idea where a
    // cross toolchain keeps newlib headers (inttypes.h, stdint.h, ...).
    // LVGL_SYSROOT points at the target sysroot (e.g. the riscv32-esp-elf
    // one printed by `riscv32-esp-elf-gcc -print-sysroot`).
    if let Ok(sysroot) = env::var("LVGL_SYSROOT") {
        if !sysroot.is_empty() {
            cc_args.push(format!("-I{}", Path::new(&sysroot).join("include").display()));
        }
    }

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mut bindings_builder = bindgen::Builder::default()
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
        .allowlist_var("LV_.*")
        // Built-in fonts are extern `lv_font_t` variables (lowercase), e.g.
        // lv_font_montserrat_14; expose the ones enabled in lv_conf.h.
        .allowlist_var("lv_font_.*")
        .blocklist_function("lv_log_add"); // varargs (va_list); not safely bindable on all targets
    // The public lvgl.h tree doesn't declare demo entry points; parse the
    // demo header too so lv_demo_widgets() lands in the bindings.
    if demo_widgets {
        bindings_builder =
            bindings_builder.header(lvgl_root.join("demos/widgets/lv_demo_widgets.h").to_str().unwrap());
    }
    let bindings = bindings_builder
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

impl SdlFlags {
    fn empty() -> Self {
        SdlFlags {
            include_dirs: Vec::new(),
            defines: Vec::new(),
            link_dirs: Vec::new(),
            libs: Vec::new(),
            frameworks: Vec::new(),
        }
    }
}

fn discover_sdl2() -> Option<SdlFlags> {
    use std::process::Command;

    fn probe(tool: &str, arg: &str) -> Option<String> {
        let out = Command::new(tool).arg(arg).output().ok()?;
        if !out.status.success() {
            return None;
        }
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }

    fn absorb(out: &str, flags: &mut SdlFlags) {
        let mut fw_next = false;
        for tok in out.split_whitespace() {
            if fw_next {
                flags.frameworks.push(tok.to_string());
                fw_next = false;
            } else if let Some(dir) = tok.strip_prefix("-I") {
                if dir.is_empty() {
                    continue;
                }
                flags.include_dirs.push(dir.to_string());
                // `sdl2-config` only exposes the .../include/SDL2 leaf, while
                // LVGL includes "SDL2/SDL.h" relative to the include root.
                if Path::new(dir).file_name().and_then(|s| s.to_str()) == Some("SDL2") {
                    if let Some(parent) = Path::new(dir).parent() {
                        let parent = parent.to_string_lossy().to_string();
                        if !flags.include_dirs.contains(&parent) {
                            flags.include_dirs.push(parent);
                        }
                    }
                }
            } else if let Some(def) = tok.strip_prefix("-D") {
                let (k, v) = def.split_once('=').unwrap_or((def, ""));
                // A bare `-Dfoo` means "defined as 1" for C compilers; keep
                // the cc and bindgen args identical by normalizing here.
                let v = if v.is_empty() { "1" } else { v };
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
            } else if let Some(rest) = tok.strip_prefix("-Wl,") {
                // e.g. `-Wl,-framework,Cocoa` from pkg-config --libs
                let mut parts = rest.split(',');
                while let Some(part) = parts.next() {
                    if part == "-framework" {
                        if let Some(fw) = parts.next() {
                            flags.frameworks.push(fw.to_string());
                        }
                    }
                    // other linker flags (rpath, ...) are left to the dylib
                }
            } else if tok.starts_with('-') {
                println!("cargo:warning=lvgl-sys: ignoring unrecognized SDL2 flag `{tok}`");
            }
        }
    }

    // pkg-config is preferred, but both queries must succeed: a partial
    // result (e.g. a broken .pc) would silently drop include or link flags.
    let mut flags = SdlFlags::empty();
    let mut pkg_errs: Vec<String> = Vec::new();
    let mut pkg_ok = true;
    for arg in ["--cflags", "--libs"] {
        match probe("pkg-config", arg) {
            Some(out) => absorb(&out, &mut flags),
            None => {
                pkg_ok = false;
                pkg_errs.push(format!("pkg-config {arg} failed"));
            }
        }
    }
    if !pkg_ok {
        flags = SdlFlags::empty();
        let cfg_ok = probe("sdl2-config", "--cflags").is_some_and(|out| {
            absorb(&out, &mut flags);
            true
        });
        let libs_ok = probe("sdl2-config", "--libs").is_some_and(|out| {
            absorb(&out, &mut flags);
            true
        });
        if !(cfg_ok && libs_ok) {
            for e in &pkg_errs {
                println!("cargo:warning=lvgl-sys: {e}");
            }
            println!("cargo:warning=lvgl-sys: sdl2-config fallback failed; is SDL2 installed?");
            return None;
        }
    }
    Some(flags)
}

/// Patch the vendored lv_conf.h for SDL simulator builds: SDL backend on,
/// C allocator (the 64 kB embedded TLSF pool cannot host desktop-size
/// snapshot buffers), warnings to stdout. Textual patching against the
/// pinned vendored file keeps default builds untouched.
fn patch_lv_conf_for_sdl(base: &str) -> String {
    let edits = [
        ("#define LV_USE_SDL 0", "#define LV_USE_SDL 1"),
        (
            "#define LV_USE_STDLIB_MALLOC LV_STDLIB_BUILTIN",
            "#define LV_USE_STDLIB_MALLOC LV_STDLIB_CLIB",
        ),
        ("#define LV_USE_LOG 0", "#define LV_USE_LOG 1"),
        ("#define LV_LOG_PRINTF 0", "#define LV_LOG_PRINTF 1"),
    ];
    let mut s = base.to_string();
    for (from, to) in edits {
        assert!(
            s.contains(from),
            "lv_conf.h no longer contains `{from}`; update patch_lv_conf_for_sdl"
        );
        s = s.replace(from, to);
    }
    s
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
