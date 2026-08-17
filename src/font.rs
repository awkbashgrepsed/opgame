use fontdue::Font;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Clone, Copy)]
pub struct GlyphInfo {
    pub u0: f32,
    pub v0: f32,
    pub u1: f32,
    pub v1: f32,
    pub width: f32,
    pub height: f32,
    pub advance: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

pub struct FontRenderer {
    font: Font,
    size: f32,
    atlas: Vec<u8>,
    atlas_width: usize,
    atlas_height: usize,
    pen_x: usize,
    pen_y: usize,
    row_height: usize,
    glyphs: HashMap<char, GlyphInfo>,
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
        Ok(Self {
            font,
            size,
            atlas: vec![0; 1024 * 1024],
            atlas_width: 1024,
            atlas_height: 1024,
            pen_x: 1,
            pen_y: 1,
            row_height: 0,
            glyphs: HashMap::new(),
        })
    }

    pub fn size(&self) -> f32 { self.size }

    pub fn rasterize_glyph(&mut self, ch: char) -> Option<(GlyphInfo, Vec<u8>)> {
        if let Some(info) = self.glyphs.get(&ch).copied() {
            return Some((info, Vec::new()));
        }
        let (metrics, bitmap) = self.font.rasterize(ch, self.size);
        let w = metrics.width.max(1);
        let h = metrics.height.max(1);
        if self.pen_x + w + 1 >= self.atlas_width {
            self.pen_x = 1;
            self.pen_y += self.row_height + 1;
            self.row_height = 0;
        }
        if self.pen_y + h + 1 >= self.atlas_height { return None; }
        for y in 0..h {
            let dst = (self.pen_y + y) * self.atlas_width + self.pen_x;
            let src = y * w;
            self.atlas[dst..dst + w].copy_from_slice(&bitmap[src..src + w]);
        }
        let info = GlyphInfo {
            u0: self.pen_x as f32 / self.atlas_width as f32,
            v0: self.pen_y as f32 / self.atlas_height as f32,
            u1: (self.pen_x + w) as f32 / self.atlas_width as f32,
            v1: (self.pen_y + h) as f32 / self.atlas_height as f32,
            width: w as f32,
            height: h as f32,
            advance: metrics.advance_width,
            offset_x: metrics.xmin as f32,
            offset_y: metrics.ymin as f32,
        };
        self.pen_x += w + 1;
        self.row_height = self.row_height.max(h);
        self.glyphs.insert(ch, info);
        Some((info, bitmap))
    }

    pub fn atlas(&self) -> &[u8] { &self.atlas }
    pub fn glyph(&self, ch: char) -> Option<GlyphInfo> { self.glyphs.get(&ch).copied() }
}
