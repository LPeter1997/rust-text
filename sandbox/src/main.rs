use std::io::prelude::*;
use std::fs::File;
use std::collections::HashMap;
use rust_text as rt;

fn main() {
    let file = File::open("Hack-Regular.ttf").expect("couldn't find font file");
    let bytes = file.bytes().map(|b| b.unwrap()).collect::<Vec<_>>().into_boxed_slice();

    let f = rt::Font::from_bytes(&bytes).expect("couldn't load font");

    for face in f.get_face_names() {
        println!("Face: {}", face);
    }

    let f0 = f.get_face(&*f.get_face_names()[0]).expect("couldn't load face from font");
    let mut sf = f0.scale(16.0, 96.0).expect("couldn't scale font");

    /*
    let ch1 = sf.rasterize_glyph('A').expect("couldn't render glyph");
    image::save_buffer("image1.png", &ch1.data, ch1.width as u32, ch1.height as u32, image::Gray(8)).expect("failed to write image");
    let ch2 = sf.rasterize_glyph('$').expect("couldn't render glyph");
    image::save_buffer("image2.png", &ch2.data, ch2.width as u32, ch2.height as u32, image::Gray(8)).expect("failed to write image");
    */

    let text = "Hello, World!\nBye World!";
    let text_len = text.chars().count();

    let mut glyphs = HashMap::new();
    for c in text.chars() {
        glyphs.insert(c, sf.rasterize_glyph(c).expect("Failed to render glyph!"));
    }
    glyphs.insert('_', sf.rasterize_glyph('_').expect("Failed to render glyph!"));

    let (mut w, mut h) = sf.shape_text(text, |_| {});
    let text_height = h;
    w += 50;
    h += h * text_len as i32;
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

    let mut yoff = 0;
    sf.shape_text(text, |p| {
        let glyph = glyphs.get(&p.character).unwrap();
        blit(p.x + glyph.x_offset, p.y + glyph.y_offset + yoff, glyph);
    });

    for i in 0..text_len {
        yoff += text_height;
        sf.shape_text(text, |p| {
            let glyph = glyphs.get(&p.character).unwrap();
            blit(p.x + glyph.x_offset, p.y + glyph.y_offset + yoff, glyph);

            if i == p.index {
                let glyph = glyphs.get(&'_').unwrap();
                blit(p.caret_x + glyph.x_offset, p.caret_y + glyph.y_offset + yoff, glyph);
            }
        });
    }

    image::save_buffer("out.png", &buff, w as u32, h as u32, image::Gray(8)).expect("failed to write image");
}
