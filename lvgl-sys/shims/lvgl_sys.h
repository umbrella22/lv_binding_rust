#ifndef LVGL_API_H
#define LVGL_API_H

#ifdef __cplusplus
extern "C" {
#endif

/* LVGL 9.2 main header (vendored at vendor/lvgl-9.2/lvgl.h, on the include
 * path through build.rs). */
#include "lvgl.h"

/* In LVGL 8 the color access macros were the canonical accessor surface;
 * LVGL 9 stores native RGB888 directly in lv_color_t.{red,green,blue}.
 * These shims keep the historical Rust call sites compiling regardless of
 * how the underlying color storage evolves. Alpha lives in lv_color32_t in
 * LVGL 9 -- _LV_COLOR_GET_A always returns 0xFF for the opaque lv_color_t. */
lv_color_t _LV_COLOR_MAKE(uint8_t r, uint8_t g, uint8_t b);
uint16_t _LV_COLOR_GET_R(lv_color_t color);
uint16_t _LV_COLOR_GET_G(lv_color_t color);
uint16_t _LV_COLOR_GET_B(lv_color_t color);
uint16_t _LV_COLOR_GET_A(lv_color_t color);

#ifdef __cplusplus
} /* extern "C" */
#endif

#endif /*LVGL_API*/
