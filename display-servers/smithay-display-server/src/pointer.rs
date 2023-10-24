use std::{io::Read, time::Duration};

use xcursor::{
    parser::{parse_xcursor, Image},
    CursorTheme,
};

static FALLBACK_CURSOR_DATA: &[u8] = include_bytes!("../resources/cursor.rgba");

pub struct Cursor {
    icons: Vec<Image>,
    size: u32,
}

impl Cursor {
    pub fn load() -> Cursor {
        let name = std::env::var("XCURSOR_THEME")
            .ok()
            .unwrap_or_else(|| "default".into());
        let size = std::env::var("XCURSOR_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(24);

        let theme = CursorTheme::load(&name);
        let icons = load_icon(&theme).unwrap_or_else(|| {
            vec![Image {
                size: 32,
                width: 64,
                height: 64,
                xhot: 1,
                yhot: 1,
                delay: 1,
                pixels_rgba: Vec::from(FALLBACK_CURSOR_DATA),
                pixels_argb: vec![], //unused
            }]
        });

        Cursor { icons, size }
    }

    pub fn get_image(&self, scale: u32, time: Duration) -> Image {
        let size = self.size * scale;
        frame(time.as_millis() as u32, size, &self.icons)
    }
}

fn nearest_images(size: u32, images: &[Image]) -> impl Iterator<Item = &Image> {
    // Follow the nominal size of the cursor to choose the nearest
    let nearest_image = images
        .iter()
        .min_by_key(|image| (size as i32 - image.size as i32).abs())
        .unwrap();

    images.iter().filter(move |image| {
        image.width == nearest_image.width && image.height == nearest_image.height
    })
}

fn frame(mut millis: u32, size: u32, images: &[Image]) -> Image {
    let total = nearest_images(size, images).fold(0, |acc, image| acc + image.delay);
    millis %= total;

    for img in nearest_images(size, images) {
        if millis < img.delay {
            return img.clone();
        }
        millis -= img.delay;
    }

    unreachable!()
}

// TODO: Sensible error handling instead of a silent return
fn load_icon(theme: &CursorTheme) -> Option<Vec<Image>> {
    let icon_path = theme.load_icon("default")?;
    let mut cursor_file = std::fs::File::open(icon_path).ok()?;
    let mut cursor_data = Vec::new();
    cursor_file.read_to_end(&mut cursor_data).ok()?;
    parse_xcursor(&cursor_data)
}
