## Examples of how to use various  `lvgl-rs` widgets/components

All examples can be executed using:
```shell
cargo run --example <name> --features="alloc"
```
while in the `lvgl-rs` directory (i.e. one up from this). Most examples
draw through an [`embedded-graphics`](https://docs.rs/embedded-graphics)
simulator window; `sdl_demo` opens LVGL's native SDL2 window instead and
needs the `sdl` feature:
```shell
cargo run -p lvgl --example sdl_demo --features sdl
```
