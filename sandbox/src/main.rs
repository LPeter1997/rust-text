use std::io::prelude::*;
use std::fs::File;
use std::collections::HashMap;
use rust_text as rt;

fn main() {
    let file = File::open("Arial.ttf").expect("couldn't find font file");
    let bytes = file.bytes().map(|b| b.unwrap()).collect::<Vec<_>>().into_boxed_slice();

    let f = rt::Font::from_bytes(&bytes).expect("couldn't load font");

    for face in f.get_face_names() {
        println!("Face: {}", face);
    }

    let f0 = f.get_face(&*f.get_face_names()[0]).expect("couldn't load face from font");
    let mut sf = f0.scale(24.0, 96.0).expect("couldn't scale font");

    let text = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789.,:;!?-+=";
    let text_len = text.chars().count();

    let mut glyphs = HashMap::new();
    for c in text.chars() {
        glyphs.insert(c, sf.rasterize_glyph(c).expect("Failed to render glyph!"));
    }
    glyphs.insert('_', sf.rasterize_glyph('_').expect("Failed to render glyph!"));

    let pack = rt::pack_glyphs(glyphs.values());
    let w = pack.width as i32;
    let h = pack.height as i32;
    let mut buff = vec![0u8; (w * h).abs() as usize];

    let mut blit = |x0: i32, y0: i32, rg: &rt::RasterizedGlyph| {
        for y in 0..rg.height {
            for x in 0..rg.width {
                let pix = rg.data[y * rg.width + x];
                if pix != 0 {
                    let idx = ((y0 + y as i32) * w + x0 + x as i32) as usize;
                    if idx < buff.len() {
                        buff[idx] = pix;
                    }
                }
            }
        }
    };

    for (c, rect) in pack.items {
        let g = glyphs.get(&c).unwrap();
        blit(rect.x as i32, rect.y as i32, g);
    }

    image::save_buffer("out.png", &buff, w as u32, h as u32, image::Gray(8)).expect("failed to write image");
}
