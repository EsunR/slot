#![cfg(feature = "rg35xxsp")]

use slot_gfx::{fit_rect, System, OUT_H, OUT_W};

#[test]
fn rg35xxsp_renders_at_native_panel_resolution() {
    assert_eq!((OUT_W, OUT_H), (640, 480));
    assert_eq!(fit_rect(640, 480), (0, 0, 640, 480));
}

#[test]
fn games_use_the_expected_rg35xxsp_viewports() {
    // Match the viewport supplied with the user's RG35XXSP GBA bezel.
    assert_eq!(System::Gba.screen_base(), (0.0, 0.0, 640.0, 426.0));

    assert_eq!(System::Gb.screen_base(), (80.0, 24.0, 480.0, 432.0));
    assert_eq!(System::Gba.mask_tiles(), (640.0 / 3.0, 142.0));
    assert_eq!(System::Gb.mask_tiles(), (160.0, 144.0));
}
