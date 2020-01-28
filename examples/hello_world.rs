
// An example that writes "Hello, World!" text to the file "hello_world.png".

use std::collections::HashMap;
mod common;
use common::*;
use rust_text as rt;

fn main() {
    // Set up paths to the used font and the output image.
    let examples_path = format!("{}/examples", env!("CARGO_MANIFEST_DIR"));
    let font_path = format!("{}/JetBrainsMono-Regular.ttf", examples_path);
    let out_path = format!("{}/hello_world.png", examples_path);
    // Load the bytes of the font file. Not part of the API.
    let bytes = load_bytes(&font_path);

    // The text we want to print.
    let text = "Hello, World!";

    // Parse the loaded bytes of the font file.
    let font = rt::Font::from_bytes(&bytes).expect("Failed to parse font!");

    // We list each face in the font file, as there is a possibility that there
    // are multiple faces in a single font file.
    for face_name in font.face_names() {
        println!("Face: {}", face_name);
    }

    // Pick the first font face from the font file.
    let font_face = font.face(font.face_names()[0].as_ref()).expect("Failed to get font face!");
    // Scale the face to 24 pts on a 96 DPI display.
    let mut scaled_face = font_face.scale(24.0, 96.0).expect("Failed to scale font!");

    // Rasterize each required glyph and create a map from character to rendered glyph.
    let glyph_lut: HashMap<_, _> = text.chars()
        .map(|c| (c, scaled_face.rasterize_glyph(c).expect("Failed to rasterize glyph!")))
        .collect();

    // Measure the required dimensions by invoking shape_text without processing anything.
    // Note: In a game you probably don't need to allocate a buffer , so you would only
    // call once and draw. In this case we need to allocate a pixel buffer in advance, so
    // we invoke shape_text twice.
    let (width, height) = scaled_face.shape_text(text, rt::ShapeOptions::default(), |_| {});
    // We create the bitmap that we will write the result to. Not part of the API.
    let mut bitmap = Bitmap::new(width as usize, height as usize);
    // Invoke shape_text again, this time to actually position the glyphs.
    scaled_face.shape_text(text, rt::ShapeOptions::default(), |info| {
        // Look up the rendered glyph.
        let glyph = glyph_lut.get(&info.character).expect("Could not find glyph!");
        // Calculate the exact character placement position. The reason we need to fiddle
        // with offsets is because the buffer is always trimmed to the minimum, so the
        // rasterized glyph needs to know how much has been trimmed from it.
        let xp = (info.x + glyph.x_offset) as usize;
        let yp = (info.y + glyph.y_offset) as usize;
        // Draw the glyph to the given position. Not part of the API.
        bitmap.blit(xp, yp, glyph);
    });

    // Saves the bitmap. Not part of the API.
    bitmap.to_file(&out_path);
}
