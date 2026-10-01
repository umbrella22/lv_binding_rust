#include "lvgl_sys.h"

/* LVGL 9.x: lv_color_t = { uint8_t blue; uint8_t green; uint8_t red; }
 * (BGR storage, native RGB888). Alpha is only carried by lv_color32_t. */

lv_color_t _LV_COLOR_MAKE(uint8_t r, uint8_t g, uint8_t b)
{
    return lv_color_make(r, g, b);
}

uint16_t _LV_COLOR_GET_R(lv_color_t color)
{
    return color.red;
}

uint16_t _LV_COLOR_GET_G(lv_color_t color)
{
    return color.green;
}

uint16_t _LV_COLOR_GET_B(lv_color_t color)
{
    return color.blue;
}

uint16_t _LV_COLOR_GET_A(lv_color_t color)
{
    (void)color;
    return 0xFF; /* lv_color_t is fully opaque in LVGL 9 */
}
