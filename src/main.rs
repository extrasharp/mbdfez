// mbdfez

// load bdf font,
// just glyphs are in a png file
//    with tag for unicode codepoint

use std::{
    convert::TryFrom,
    fs,
};

use bdf_font::*;
use image::{
    self,
    RgbaImage,
    Rgba,
    GenericImageView,
};

fn rgba_as_u32(rgba: &Rgba<u8>) -> u32 {
    (rgba.0[0] as u32) << 16 |
    (rgba.0[1] as u32) << 8 |
    (rgba.0[2] as u32)
}

struct GlyphsSheet {
    marker: Rgba<u8>,
    draw_color: Rgba<u8>,
    grid_width: u32,
    grid_height: u32,
    glyphs: Vec<Glyph>,
}

impl GlyphsSheet {
    fn new_parse(img: &RgbaImage) -> Self {
        let marker = img.get_pixel(0, 0);
        let draw_color = img.get_pixel(1, 0);
        let grid_width = rgba_as_u32(img.get_pixel(3, 0));
        let grid_height = rgba_as_u32(img.get_pixel(4, 0));

        let x_offset = 1;
        let y_offset = 1;
        let glyph_width = grid_width - x_offset;
        let glyph_height = grid_height - y_offset;

        let mut glyphs = Vec::new();

        let w_ct = img.width() / grid_width;
        let h_ct = img.height() / grid_height;

        for i in 0..w_ct {
            for j in 1..h_ct {
                let view = img.view(i * grid_width, j * grid_height, grid_width, grid_height);
                if view.get_pixel(0, 0) == *marker {
                    let codepoint_high = rgba_as_u32(&view.get_pixel(1, 0));
                    let codepoint_low = rgba_as_u32(&view.get_pixel(2, 0));
                    let codepoint = ((codepoint_high & 0xFF) << 8) |
                                    (codepoint_low & 0xFF);

                    let glyph_view = view.view(x_offset, y_offset, glyph_width, glyph_height);
                    let mut bitmap = Bitmap::new(glyph_width as usize, glyph_height as usize);
                    for (x, y, p) in glyph_view.pixels() {
                        bitmap.set(x as usize, y as usize, p == *draw_color);
                    }

                    // TODO calulate bounding box based on how much is covered by glyph
                    // TODO get metrics

                    glyphs.push(Glyph {
                        name: format!("U+{:04X}", codepoint),
                        codepoint: char::try_from(codepoint).unwrap(),
                        bounding_box: BoundingBox::new(glyph_width, glyph_height, 0, 0),
                        bitmap,
                        metrics: MetricsSet::Normal,
                        scalable_width: None,
                        device_width: None,
                        scalable_width_alt: None,
                        device_width_alt: None,
                        vector: None,
                    });
                }
            }
        }

        let marker = *marker;
        let draw_color = *draw_color;

        Self {
            marker,
            draw_color,
            grid_width,
            grid_height,
            glyphs,
        }
    }
}

fn main() {
    let filename = "src/test_font";
    let img = image::open([filename, ".png"].concat()).unwrap().to_rgba();
    let gs = GlyphsSheet::new_parse(&img);

    let s = fs::read_to_string([filename, ".bdf"].concat()).unwrap();
    let mut fnt = parse_font(&s).unwrap();
    fnt.glyphs = gs.glyphs;

    println!("{:#?}", fnt);
}
