# lvgl-sys
Rust raw bindings for the LVGL C library.

## Usage

No configuration is required: the build compiles the vendored LVGL 9.6
source tree (`vendor/lvgl-9.6/`) against the vendored `lv_conf.h`
(`vendor/include-9.6/lv_conf.h`). Edit that header to enable or disable
LVGL features; everything it enables is picked up by the generated
bindings.

```shell script
$ cargo build
```

The optional `LVGL_CFLAGS` environment variable forwards extra C compiler
definitions (comma-separated `NAME=VALUE` pairs) to both the C build and
bindgen.

The `sdl` cargo feature additionally compiles and links LVGL's native SDL2
backend (`LV_USE_SDL`) for desktop simulation; it requires SDL2 on the host
and is never enabled by default, so embedded/ESP cross-builds don't need
SDL installed.
