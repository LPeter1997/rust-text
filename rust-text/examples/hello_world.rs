
// An example that writes "Hello, World!" text to the file "hello_world.png".

mod common;
use common::*;
use rust_text as rt;

fn main() {
    println!("{}", env!("CARGO_MANIFEST_DIR"));

    let bytes = load_bytes("JetBrainsMono-Regular.ttf");
    let font = rt::Font::from_bytes(&bytes).expect("Failed to parse font!");

    for face_name in font.face_names() {
        println!("Face: {}", face_name);
    }

    let font_face = font.face(font.face_names()[0].as_ref()).expect("Failed to get font face!");
    let mut scaled_face = font_face.scale(24.0, 96.0).expect("Failed to scale font!");

    let mut bitmap = Bitmap::new(10, 10);
}
