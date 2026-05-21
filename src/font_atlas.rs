use crate::{buffers::Texture, context::Context};
use ab_glyph::{Font, FontRef, PxScale};

pub struct FontAtlas {
    pub texture: Texture,
}

impl FontAtlas {
    pub fn new(context: &mut Context, chars: &str, font: &str, font_size: f32) -> Self {
        let font = FontRef::try_from_slice(include_bytes!("../JetBrainsMono.ttf"))
            .expect("Failed to load font from bytes");
        let scale = PxScale::from(font_size);

        let mut max_width = 0.0_f32;
        let mut max_height = 0.0_f32;

        for c in chars.chars() {
            let glyph = font.glyph_id(c).with_scale(scale);
            if let Some(outlined) = font.outline_glyph(glyph) {
                let bounds = outlined.px_bounds();
                max_width = max_width.max(bounds.max.x - bounds.min.x);
                max_height = max_height.max(bounds.max.y - bounds.min.y);
            }
        }

        let cell_width = max_width.ceil() as usize;
        let cell_height = max_height.ceil() as usize;
        let columns = 8;
        let rows = (chars.len() + columns - 1) / columns;

        log::info!("Creating font atlas with {} characters, {} columns, {} rows, cell size {}x{}", chars.len(), columns, rows, cell_width, cell_height);

        let width = columns * cell_width;
        let height = rows * cell_height;
        let components = 4;
        let mut atlas_data = vec![0u8; width * height * components];

        for (i, c) in chars.chars().enumerate() {
            let glyph = font.glyph_id(c).with_scale(scale);
            if let Some(outlined) = font.outline_glyph(glyph) {
                let x_offset = (i % columns) * cell_width;
                let y_offset = (i / columns) * cell_height;

                let bounds = outlined.px_bounds();
                let char_width = bounds.max.x - bounds.min.x;
                let char_height = bounds.max.y - bounds.min.y;
                
                let center_x = ((cell_width as f32 - char_width) / 2.0).max(0.0) as usize;
                let center_y = ((cell_height as f32 - char_height) / 2.0).max(0.0) as usize;

                outlined.draw(|x, y, v| {
                    let final_x = x_offset + center_x + x as usize;
                    let final_y = y_offset + center_y + y as usize;
                    
                    if final_x < width && final_y < height {
                        let px = (final_x + final_y * width) * components;
                        let color = (v * 255.0) as u8;
                        atlas_data[px] = color;
                        atlas_data[px + 1] = color;
                        atlas_data[px + 2] = color;
                        atlas_data[px + 3] = color;
                    }
                });
            }
        }

        let texture = Texture::new(
            width as u32,
            height as u32,
            context,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            Some("Font Atlas Texture"),
        );

        context.queue.write_texture(
            texture.texture.as_image_copy(),
            &atlas_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some((width * components) as u32),
                rows_per_image: None,
            },
            texture.extent,
        );

        Self { texture }
    }
}
