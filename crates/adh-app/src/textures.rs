//! Décodage et cache des visuels embarqués.
//!
//! Les images sont fournies par `adh-assets` sous forme de bytes PNG/JPEG. Elles
//! sont décodées une seule fois avec la crate `image`, puis confiées à egui. Le
//! RGBA est conservé pour les médiators : l'application d'origine n'acceptait un
//! clic que si le pixel touché n'était pas transparent.

use std::collections::HashMap;

use egui::{ColorImage, Context, TextureHandle, TextureOptions};
use image::ImageReader;

/// Une image décodée.
pub struct Decoded {
    /// Largeur en pixels.
    pub width: u32,
    /// Hauteur en pixels.
    pub height: u32,
    /// Pixels RGBA.
    pub rgba: Vec<u8>,
    /// Version egui.
    pub color: ColorImage,
}

impl Decoded {
    fn blank() -> Self {
        Self {
            width: 1,
            height: 1,
            rgba: vec![0, 0, 0, 0],
            color: ColorImage::from_rgba_unmultiplied([1, 1], &[0, 0, 0, 0]),
        }
    }

    /// Canal alpha du pixel `(x, y)`, `0` hors de l'image.
    pub fn alpha_at(&self, x: u32, y: u32) -> u8 {
        if x >= self.width || y >= self.height {
            return 0;
        }
        let index = ((y * self.width + x) as usize) * 4;
        self.rgba.get(index + 3).copied().unwrap_or(0)
    }
}

/// Décode des bytes PNG ou JPEG.
pub fn decode(bytes: &[u8]) -> Result<Decoded, String> {
    let reader = ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|error| error.to_string())?;
    let image = reader.decode().map_err(|error| error.to_string())?;
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    let color = ColorImage::from_rgba_unmultiplied(
        [width as usize, height as usize],
        rgba.as_raw(),
    );
    Ok(Decoded {
        width,
        height,
        rgba: rgba.into_raw(),
        color,
    })
}

/// Cache d'images décodées et de textures egui.
pub struct Store {
    decoded: HashMap<String, Decoded>,
    textures: HashMap<String, TextureHandle>,
}

impl Store {
    /// Cache vide.
    pub fn new() -> Self {
        Self {
            decoded: HashMap::new(),
            textures: HashMap::new(),
        }
    }

    /// Bytes décodés d'une image, décodés à la première demande.
    pub fn decoded(&mut self, key: &str, bytes: &[u8]) -> &Decoded {
        if !self.decoded.contains_key(key) {
            let entry = decode(bytes).unwrap_or_else(|error| {
                eprintln!("image {} illisible : {}", key, error);
                Decoded::blank()
            });
            self.decoded.insert(key.to_string(), entry);
        }
        self.decoded.get(key).expect("entrée ajoutée")
    }

    /// Texture egui d'une image, créée à la première demande.
    pub fn texture(&mut self, ctx: &Context, key: &str, bytes: &[u8]) -> TextureHandle {
        if let Some(texture) = self.textures.get(key) {
            return texture.clone();
        }
        let color = self.decoded(key, bytes).color.clone();
        let texture = ctx.load_texture(key, color, TextureOptions::LINEAR);
        self.textures.insert(key.to_string(), texture.clone());
        texture
    }

    /// Oublie les textures (utile au changement d'instrument).
    pub fn forget_textures(&mut self) {
        self.textures.clear();
    }
}
