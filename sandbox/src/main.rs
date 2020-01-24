use rust_text as rt;

fn main() {
    let f = rt::Font::from_bytes().expect("couldn't load font");
    let mut sf = f.scale().expect("couldn't scale font");
    let ch1 = sf.render_glyph('A').expect("couldn't render glyph");
    image::save_buffer("image1.png", &ch1.data, ch1.width as u32, ch1.height as u32, image::Gray(8));
    let ch2 = sf.render_glyph('$').expect("couldn't render glyph");
    image::save_buffer("image2.png", &ch2.data, ch2.width as u32, ch2.height as u32, image::Gray(8));
}
