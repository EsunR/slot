//! One machine's picture geometry: how big a frame its core hands us, and where on the
//! panel that frame lands.
//!
//! The default panel is 720x480, where GBA's 240x160 frame fits at exactly 3x. Smaller
//! panels fit the game frame within their bounds while keeping its aspect ratio.
//!
//! Game Boy is 160x144 and fits at 3x on both panels: a 480x432 window centred in the frame.
//! That window is what a bezel goes around.
//!
//! Game Boy and Game Boy Color get one row between them rather than one each: their frames
//! are the same 160x144, so they want the same texture and the same window, and a second copy
//! would only be a second place for the two to drift apart.

use crate::surface::{OUT_H, OUT_W};

/// Maximum game scale. The default panel reaches exactly 3x; smaller panels may use less.
pub const SCALE: u32 = 3;

/// The GBA's own frame. The pair this module used to be.
pub const GBA_SRC_W: u32 = 240;
pub const GBA_SRC_H: u32 = 160;

/// Game Boy's frame, and Game Boy Color's.
pub const GB_SRC_W: u32 = 160;
pub const GB_SRC_H: u32 = 144;

/// Which machine's picture is on the panel.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum System {
    Gba,
    /// Game Boy *and* Game Boy Color: one row, because one frame size.
    Gb,
}

impl System {
    /// Number of 3-pixel mask cells across the physical game viewport. At 3x this
    /// equals the source size; the RG35XXSP GBA viewport is fractional-scale, so
    /// screen-space cells keep the LCD pattern a regular size on the panel.
    pub fn mask_tiles(self) -> (f32, f32) {
        let (_, _, w, h) = self.screen_base();
        (w / 3.0, h / 3.0)
    }

    /// The frame the core hands us, in pixels. The 3x3 mask tiles once per source pixel.
    pub fn src_w(self) -> u32 {
        match self {
            System::Gba => GBA_SRC_W,
            System::Gb => GB_SRC_W,
        }
    }

    pub fn src_h(self) -> u32 {
        match self {
            System::Gba => GBA_SRC_H,
            System::Gb => GB_SRC_H,
        }
    }

    /// The rect the picture fills with the screen fully up, in offscreen pixels.
    ///
    /// The default 720x480 panel displays both systems at exactly 3x. On RG35XXSP the GBA
    /// picture uses the viewport specified by its supplied bezel (`default.cfg`): 640x426 at
    /// the top left. The PNG covers the outer eight pixels of that viewport. GB remains
    /// centred at 480x432.
    pub fn screen_base(self) -> (f32, f32, f32, f32) {
        #[cfg(feature = "rg35xxsp")]
        if self == System::Gba {
            return (0.0, 0.0, 640.0, 426.0);
        }
        let scale = (OUT_W as f32 / self.src_w() as f32)
            .min(OUT_H as f32 / self.src_h() as f32)
            .min(SCALE as f32);
        let w = self.src_w() as f32 * scale;
        let h = self.src_h() as f32 * scale;
        ((OUT_W as f32 - w) / 2.0, (OUT_H as f32 - h) / 2.0, w, h)
    }
}
