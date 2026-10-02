# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Native LVGL SDL2 desktop simulation through the `sdl` feature and the
  `sdl_demo` example.
- The official LVGL widgets demo through `lvgl/demo-widgets` and
  `lvgl-sys/lv-demo-widgets`, including its required sources and assets.
- Explicit `unsafe Display::delete()` for removing a registered native display.
- Regression tests for display lifetimes, partial and padded refresh buffers,
  generated pointer APIs, timer reinitialization, SDL configuration, and demo
  package contents.

### Changed

- **Breaking:** Migrate the Rust bindings to LVGL 9 and upgrade the vendored LVGL sources to
  9.6.0.
- Build against the vendored configuration in `lvgl-sys/vendor/include-9.6/`,
  with consistent configuration and `LVGL_CFLAGS` definitions for the C compiler
  and bindgen.
- Use the C library allocator in the vendored LVGL configuration.
- **Breaking:** Generated methods accepting raw pointer arguments now require
  `unsafe` and document the corresponding C API memory and lifetime requirements.
  Returning a raw pointer alone does not make a method unsafe.
- **Breaking:** Dropping a `Display` handle no longer deletes the native display.
  Registered callbacks and draw buffers are released when LVGL deletes the
  display, including through `unsafe Display::delete()` or `unsafe deinit()`.
  Raw cleanup hooks run during native deletion, before screen destruction.
  Deletion requires all rendering and transfers to finish, no subsequent use of
  associated handles, and no reentrant use from deletion callbacks.
- **Breaking:** Safe `Display::register` renders in RGB888 independently of the
  configured default color format, with `N` denoting the maximum refresh pixel
  count. `register_raw` retains the configured format and allocates
  `N * size_of::<lv_color_t>()` bytes, which need not hold exactly `N` pixels.

### Fixed

- Decode only the current refresh area using the actual row stride, skipping
  padding and unused buffer capacity. Reject invalid display dimensions and
  buffers too small for one aligned display row.
- Preserve default-display aliases and implicitly created widgets after a Rust
  display handle is dropped, and release registered resources on native deletion.
- Run contained value destructors before freeing allocations made by the
  internal LVGL-backed `Box`.
- Scale RGB888 channels correctly when converting colors to RGB565.
- Register the Rust clock through LVGL 9's runtime tick callback, restore it after
  deinitialization and reinitialization, and keep `tick_inc` available with the
  timer features enabled.
- Make SDL configuration patches idempotent and accept configurations already
  using the C library allocator.
- Correct SDL2 include and linker flag handling, discovery fallback, and build
  configuration change tracking.
- Include the widgets demo's required files in published crates so the demo
  feature also builds from an unpacked package.
- Update examples, font documentation, and CI checks for the LVGL 9 build and
  native SDL2 simulator.

### Removed

- Legacy LVGL 8 external configuration, library, and font auto-discovery through
  `DEP_LV_CONFIG_PATH`, `LVGL_INCLUDE`, `LVGL_LINK`, and `LVGL_FONTS_DIR`.
- The legacy `lv_drivers` backend integration; the `drivers` feature remains as
  a compatibility no-op in `lvgl-sys`. Native desktop simulation uses `sdl`.

## [0.6.2]

### Fixed

- Fix build in docs.rs

## [0.6.1]

### Fixed

- Excluded LVGL demos due to crates.io binary size limits

## [0.6.0]

### Added

- Support pointer input devices #62
- Enable interop with [`lv_drivers`](https://github.com/lvgl/lv_drivers) #64
- Allow using both custom and built-in fonts #76
- Support setting the LVGL timer from Rust #81
- Support building from Windows #55
- Enable using the vendored LVGL config #56
- Allow screen switching #57
- Add examples for #64 and #81
- Add a lot of documentation

### Fixed

- Example README now properly specifies how to run
- No more undefined behavior if LVGL is not properly initialized
- Fixed various miscompilations and counts of undefined behavior

### Changed

- Changed core API entirely #51
- Updated LVGL to 8.3.5 #67
- Updated dependencies #61

### Removed

- The `UI` struct and its related API

## [0.5.2] - 2021-03-06

### Added

- Expose RGB values from Color #29
- Make lvgl possible to compile to WASM using Emscripten #31 (complete example available at [lvgl-rs-wasm](https://github.com/rafaelcaricio/lvgl-rs-wasm) and [live](https://rafaelcaricio.github.io/lvgl-rs-wasm/) on the web)

### Fixed

- Fix documentation generation, now we will be visible in docs.rs website 🥳 #41 
- Fix compiler error when running the examples #40

### Changed

- Updated README:
  - Added a hint to install SDL2 before running the demos on macOS #36
  - Add system dependencies for compilation #41

## [0.4.0] - 2020-06-19

### Changed

- Simplify examples by removing the use of threads

### Removed

- Removes the dependency on `alloc` crate

## [0.3.1] - 2020-06-14

### Changed

- Replace `string.c` with implementation in Rust

## [0.3.0] - 2020-06-14

### Added

- New code generation for the safe bindings based on the [`syn`](https://docs.rs/syn/1.0.31/syn/index.html) crate. This uses `lvgl-codegen` directly, which implements code generation for known patterns. This avoids a lot of manual work to expose LVGL API as safe Rust API

### Changed

- Code generation is completely transparent to users
- The code in `lvgl-codegen` gets cleaner and intuitive to write, since now we are processing Rust code instead of C. C is completely abstracted at the `lvgl-sys`/`rust-bindgen` level

### Removed

- No (direct) dependency on `clang-rs`

[Unreleased]: https://github.com/umbrella22/lv_binding_rust/compare/d83b374...master
[0.6.2]: https://github.com/rafaelcaricio/lvgl-rs/compare/0.6.1..0.6.2
[0.6.1]: https://github.com/rafaelcaricio/lvgl-rs/compare/0.6.0..0.6.1
[0.6.0]: https://github.com/rafaelcaricio/lvgl-rs/compare/0.5.2..0.6.0
[0.5.2]: https://github.com/rafaelcaricio/lvgl-rs/compare/0.4.0..0.5.2
[0.4.0]: https://github.com/rafaelcaricio/lvgl-rs/compare/0.3.1..0.4.0
[0.3.1]: https://github.com/rafaelcaricio/lvgl-rs/compare/0.3.0..0.3.1
[0.3.0]: https://github.com/rafaelcaricio/lvgl-rs/releases/tag/0.3.0