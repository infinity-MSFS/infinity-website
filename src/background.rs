//! The site background: brand emerald and white glow on true black.
//!
//! `BackgroundGradientAnimation` reads every colour off a `BlobPalette`, so
//! the whole look is these values. It is dark by design: the page paints white
//! text over a `bg-black/80` sheet, so a light base would leave the copy
//! unreadable.

use infinity_ui::BlobPalette;

pub const PALETTE: BlobPalette = BlobPalette {
    background_start: "rgb(4, 32, 26)",
    background_end: "rgb(0, 0, 0)",
    colors: [
        "52, 211, 153",
        "255, 255, 255",
        "16, 185, 129",
        "45, 212, 191",
        "110, 231, 183",
    ],
    size: "80%",
    blend: "hard-light",
};

/// Flat two-stop version of the palette, for the low-performance fallback.
pub fn ramp() -> String {
    format!(
        "linear-gradient(40deg, {}, {})",
        PALETTE.background_start, PALETTE.background_end
    )
}
