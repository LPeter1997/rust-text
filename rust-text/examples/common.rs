
// Common utilities for the examples.

use std::io::prelude::*;
use std::fs::File;
use rust_text as rt;

pub(crate) fn load_bytes(path: &str) -> Box<[u8]> {
    let file = File::open(path).expect("couldn't find font file");
    file.bytes().map(|b| b.unwrap()).collect::<Vec<_>>().into_boxed_slice()
}

#[derive(Clone)]
pub(crate) struct Bitmap {
    width: usize,
    height: usize,
    data: Box<[u8]>,
}

impl Bitmap {
    pub(crate) fn new(width: usize, height: usize) -> Self {
        Self{
            width, height,
            data: Vec::with_capacity(width * height).into_boxed_slice(),
        }
    }

    pub(crate) fn blit(&mut self, x0: usize, y0: usize, glyph: &rt::RasterizedGlyph) {
        for y in 0..glyph.height {
            let yoff_buff = (y0 + y) * self.width;
            let yoff_glyph = y * glyph.width;
            for x in 0..glyph.width {
                let buff_offs = yoff_buff + x0 + x;
                if buff_offs < self.data.len() {
                    self.data[buff_offs] = glyph.data[yoff_glyph + x];
                }
            }
        }
    }

    pub(crate) fn to_file(&self, path: &str) {
        image::save_buffer(path, &self.data,
            self.width as u32, self.height as u32, image::Gray(8))
            .expect("Failed to write the image!");
    }
}
