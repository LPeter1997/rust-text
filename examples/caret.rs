
// An example that draws a text many times with a caret for each character.
// Writes the result to "caret.png".

use std::collections::HashMap;
mod common;
use common::*;
use rust_text as rt;

fn main() {
    // Set up paths to the used font and the output image.
    let examples_path = format!("{}/examples", env!("CARGO_MANIFEST_DIR"));
    let font_path = format!("{}/JetBrainsMono-Regular.ttf", examples_path);
    let out_path = format!("{}/caret.png", examples_path);
    // Load the bytes of the font file. Not part of the API.
    let bytes = load_bytes(&font_path);

    // The text we want to print.
    let text = "Hello, World!";

    // Parse the loaded bytes of the font file.
    let font = rt::Font::from_bytes(&bytes).expect("Failed to parse font!");
    // Pick the first font face from the font file.
    let font_face = font.face(font.face_names()[0].as_ref()).expect("Failed to get font face!");
    // Scale the face to 24 pts on a 96 DPI display.
    let mut scaled_face = font_face.scale(24.0, 96.0).expect("Failed to scale font!");

    // Rasterize each required glyph and create a map from character to rendered glyph.
    let mut glyph_lut: HashMap<_, _> = text.chars()
        .map(|c| (c, scaled_face.rasterize_glyph(c).expect("Failed to rasterize glyph!")))
        .collect();
    // Rasterize a caret, which is the character '_' for simplicity.
    glyph_lut.insert('_', scaled_face.rasterize_glyph('_').expect("Failed to rasterize glyph!"));

    // Measure the text dimensions so we can pre-allocate the required bitmap.
    let (width, text_height) = scaled_face.shape_text(text, rt::ShapeOptions::default(), |_| {});
    // We want to put a caret under each character, so we need chars().count() times
    // the height.
    let full_height = text_height * (text.chars().count() as i32);
    // We create the bitmap that we will write the result to. Not part of the API.
    let mut bitmap = Bitmap::new(width as usize, full_height as usize);
    // Loop for each character.
    for i in 0..text.chars().count() {
        // Vertical offset for the current text instance.
        let y_offset = i * (text_height as usize);
        // Invoke shape_text again, this time to actually position the glyphs.
        scaled_face.shape_text(text, rt::ShapeOptions::default(), |info| {
            // Look up the rendered glyph.
            let glyph = glyph_lut.get(&info.character).expect("Could not find glyph!");
            // Calculate the exact character placement position.
            let xp = (info.x + glyph.x_offset) as usize;
            let yp = (info.y + glyph.y_offset) as usize + y_offset;
            // Draw the glyph to the given position. Not part of the API.
            bitmap.blit(xp, yp, glyph);
            // If this position is a caret position (info.index == i) then draw the caret.
            if i == info.index {
                // Look up the caret glyph.
                let glyph = glyph_lut.get(&'_').expect("Could not find glyph!");
                // Calculate the exact caret placement position.
                let xp = (info.caret_x + glyph.x_offset) as usize;
                let yp = (info.caret_y + glyph.y_offset) as usize + y_offset;
                // Draw the glyph to the given position. Not part of the API.
                bitmap.blit(xp, yp, glyph);
            }
        });
    }

    // Saves the bitmap. Not part of the API.
    bitmap.to_file(&out_path);
}
