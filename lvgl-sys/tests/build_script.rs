//! Regression tests for SDL configuration and the widgets demo package inputs.

// Build scripts are not Cargo test targets. Include the actual implementation
// here so these tests cannot silently diverge from the build-time patch logic.
#[allow(dead_code)]
mod build_script {
    include!("../build.rs");

    const BASE_CONF: &str = "\
#define LV_USE_SDL 0
#define LV_USE_STDLIB_MALLOC LV_STDLIB_BUILTIN
#define LV_USE_LOG 0
#define LV_LOG_PRINTF 0
#define LV_FONT_MONTSERRAT_28 1
";

    const SDL_CONF: &str = "\
#define LV_USE_SDL 1
#define LV_USE_STDLIB_MALLOC LV_STDLIB_CLIB
#define LV_USE_LOG 1
#define LV_LOG_PRINTF 1
#define LV_FONT_MONTSERRAT_28 1
";

    #[test]
    fn sdl_patch_accepts_builtin_allocator() {
        assert_eq!(patch_lv_conf_for_sdl(BASE_CONF), SDL_CONF);
    }

    #[test]
    fn sdl_patch_accepts_clib_allocator() {
        let base = BASE_CONF.replace("LV_STDLIB_BUILTIN", "LV_STDLIB_CLIB");
        assert_eq!(patch_lv_conf_for_sdl(&base), SDL_CONF);
    }

    #[test]
    fn sdl_patch_is_idempotent() {
        assert_eq!(patch_lv_conf_for_sdl(SDL_CONF), SDL_CONF);
    }

    #[test]
    fn sdl_patch_preserves_other_vendored_settings() {
        let base = include_str!("../vendor/include-9.6/lv_conf.h");
        let expected = base
            .replace("#define LV_USE_SDL 0", "#define LV_USE_SDL 1")
            .replace(
                "#define LV_USE_STDLIB_MALLOC LV_STDLIB_BUILTIN",
                "#define LV_USE_STDLIB_MALLOC LV_STDLIB_CLIB",
            )
            .replace("#define LV_USE_LOG 0", "#define LV_USE_LOG 1")
            .replace("#define LV_LOG_PRINTF 0", "#define LV_LOG_PRINTF 1");
        assert_eq!(patch_lv_conf_for_sdl(base), expected);
    }

    #[test]
    #[should_panic(expected = "LV_USE_STDLIB_MALLOC")]
    fn sdl_patch_rejects_unsupported_allocator() {
        let base = BASE_CONF.replace("LV_STDLIB_BUILTIN", "LV_STDLIB_CUSTOM");
        patch_lv_conf_for_sdl(&base);
    }

    #[test]
    #[should_panic(expected = "LV_USE_STDLIB_MALLOC")]
    fn sdl_patch_rejects_missing_allocator() {
        let base = BASE_CONF.replace("#define LV_USE_STDLIB_MALLOC LV_STDLIB_BUILTIN\n", "");
        patch_lv_conf_for_sdl(&base);
    }
}

#[test]
fn package_contains_widgets_demo_inputs() {
    use std::{collections::BTreeSet, path::Path, process::Command};

    fn collect_files(root: &Path, dir: &Path, files: &mut BTreeSet<String>) {
        for entry in std::fs::read_dir(dir).expect("read widgets demo directory") {
            let entry = entry.expect("read widgets demo entry");
            let path = entry.path();
            if entry
                .file_type()
                .expect("read widgets demo file type")
                .is_dir()
            {
                collect_files(root, &path, files);
            } else if matches!(
                path.extension().and_then(|ext| ext.to_str()),
                Some("c" | "h")
            ) {
                files.insert(
                    path.strip_prefix(root)
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .replace('\\', "/"),
                );
            }
        }
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut required = BTreeSet::from([
        "vendor/lvgl-9.6/demos/lv_demos.c".to_string(),
        "vendor/lvgl-9.6/demos/lv_demos.h".to_string(),
        "vendor/lvgl-9.6/demos/widgets/lv_demo_widgets.c".to_string(),
        "vendor/lvgl-9.6/demos/widgets/lv_demo_widgets.h".to_string(),
    ]);
    collect_files(
        root,
        &root.join("vendor/lvgl-9.6/demos/widgets"),
        &mut required,
    );

    // Listing does not build the package; give nested Cargo its own target
    // directory so this also works while the outer `cargo test` holds a lock.
    let target = std::env::temp_dir().join(format!("lvgl-sys-package-list-{}", std::process::id()));
    let output = Command::new(env!("CARGO"))
        .current_dir(root)
        .args([
            "package",
            "--list",
            "--allow-dirty",
            "--offline",
            "--locked",
        ])
        .arg("--target-dir")
        .arg(&target)
        .output()
        .expect("run cargo package --list");
    if target.exists() {
        std::fs::remove_dir_all(&target).expect("remove package-list target directory");
    }
    assert!(
        output.status.success(),
        "cargo package --list failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 package file list");
    let packaged: BTreeSet<&str> = stdout.lines().collect();
    let missing: Vec<_> = required
        .iter()
        .filter(|path| !packaged.contains(path.as_str()))
        .collect();
    assert!(
        missing.is_empty(),
        "demo inputs missing from package: {missing:?}"
    );
}
