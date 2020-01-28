
// An example that tries to pack the given glyphs into the smallest rectangular
// space possible. Writes the result to "atlas.png".
// Note that the underlying problem is NP-hard, so the algorithm is a
// best-effort one.

use std::collections::HashMap;
mod common;
use common::*;
use rust_text as rt;

fn main() {
    // Set up paths to the used font and the output image.
    let examples_path = format!("{}/examples", env!("CARGO_MANIFEST_DIR"));
    let font_path = format!("{}/JetBrainsMono-Regular.ttf", examples_path);
    let out_path = format!("{}/atlas.png", examples_path);
    // Load the bytes of the font file. Not part of the API.
    let bytes = load_bytes(&font_path);

    // The characters we want to pack.
    let characters = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789.,:;+-*/\\%!?()[]{}";

    // Parse the loaded bytes of the font file.
    let font = rt::Font::from_bytes(&bytes).expect("Failed to parse font!");
    // Pick the first font face from the font file.
    let font_face = font.face(font.face_names()[0].as_ref()).expect("Failed to get font face!");
    // Scale the face to 24 pts on a 96 DPI display.
    let mut scaled_face = font_face.scale(24.0, 96.0).expect("Failed to scale font!");

    // Rasterize each required glyph and create a map from character to rendered glyph.
    let glyph_lut: HashMap<_, _> = characters.chars()
        .map(|c| (c, scaled_face.rasterize_glyph(c).expect("Failed to rasterize glyph!")))
        .collect();

    // Pack the glyphs into the tightest space possible.
    // Note: NP-hard, best effort algorithm.
    let pack = rt::pack_glyphs(glyph_lut.values());
    // We create the bitmap that we will write the result to. Not part of the API.
    let mut bitmap = Bitmap::new(pack.width(), pack.height());
    // Go through each packed element.
    for (character, rect) in &pack {
        // Look up the rendered glyph.
        let glyph = glyph_lut.get(character).expect("Could not find glyph!");
        // Draw the glyph to the packed position
        bitmap.blit(rect.x, rect.y, glyph);
    }
    // Saves the bitmap. Not part of the API.
    bitmap.to_file(&out_path);
}
