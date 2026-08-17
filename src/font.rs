use fontdue::Font;
use std::fs;
use std::path::Path;
use std::os::raw::c_void;

#[link(name = "opengl32")]
extern "system" {
    fn glRasterPos2f(x: f32, y: f32);
    fn glPixelZoom(xfactor: f32, yfactor: f32);
    fn glDrawPixels(width: i32, height: i32, format: u32, type_: u32, pixels: *const c_void);
}

const GL_RGBA: u32 = 0x1908;
const GL_UNSIGNED_BYTE: u32 = 0x1401;

pub struct FontRenderer {
    font: Font,
    size: f32,
}

impl FontRenderer {
    pub fn new(size: f32) -> Result<Self, String> {
        let candidates = [
            "assets/fonts/SegoeUI.ttf",
            "assets/fonts/DejaVuSans.ttf",
            "C:\\Windows\\Fonts\\segoeui.ttf",
            "C:\\Windows\\Fonts\\arial.ttf",
        ];
        let path = candidates.iter().find(|p| Path::new(p).exists())
            .ok_or_else(|| "No font found. Put a TTF in assets/fonts/SegoeUI.ttf".to_string())?;
        let bytes = fs::read(path).map_err(|e| format!("Could not read font {}: {}", path, e))?;
        let font = Font::from_bytes(bytes, fontdue::FontSettings::default())
            .map_err(|e| format!("Could not load font: {}", e))?;
        Ok(Self { font, size })
    }

    pub fn draw_text(&self, text: &str, x: f32, y: f32, color: [u8; 4]) {
        let mut width = 1usize;
        let mut height = (self.size * 1.5).ceil() as usize + 4;
        let mut glyphs = Vec::with_capacity(text.chars().count());
        for ch in text.chars() {
            let (metrics, bitmap) = self.font.rasterize(ch, self.size);
            width += metrics.advance_width.ceil().max(0.0) as usize;
            height = height.max((metrics.height as i32 + self.size as i32 + 4) as usize);
            glyphs.push((metrics, bitmap));
        }

        let mut pixels = vec![0u8; width * height * 4];
        let baseline = self.size.ceil() as i32;
        let mut pen_x = 0i32;

        for (metrics, bitmap) in glyphs {
            let gx = pen_x + metrics.xmin;
            let gy = baseline - metrics.ymin - metrics.height as i32;
            for row in 0..metrics.height {
                for col in 0..metrics.width {
                    let alpha = bitmap[row * metrics.width + col];
                    if alpha == 0 { continue; }
                    let px = gx + col as i32;
                    let py = gy + row as i32;
                    if px < 0 || py < 0 || px >= width as i32 || py >= height as i32 { continue; }
                    let dst = (py as usize * width + px as usize) * 4;
                    pixels[dst] = color[0];
                    pixels[dst + 1] = color[1];
                    pixels[dst + 2] = color[2];
                    pixels[dst + 3] = ((alpha as u16 * color[3] as u16) / 255) as u8;
                }
            }
            pen_x += metrics.advance_width.round() as i32;
        }

        unsafe {
            glRasterPos2f(x, y);
            glPixelZoom(1.0, -1.0);
            glDrawPixels(
                width as i32,
                height as i32,
                GL_RGBA,
                GL_UNSIGNED_BYTE,
                pixels.as_ptr() as *const c_void,
            );
            glPixelZoom(1.0, 1.0);
        }
    }
}
