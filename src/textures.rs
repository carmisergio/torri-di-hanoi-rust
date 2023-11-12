extern crate image;

use image::*;
use std::path::*;

// Types
type RGBColor = [u8; 3];
type HSVColor = [u8; 3];

fn hsv_to_rgb(hsv: HSVColor) -> RGBColor {
    let mut h = hsv[0] as f64 / 255.0;
    let s = hsv[1] as f64 / 255.0;
    let v = hsv[2] as f64 / 255.0;

    let mut r: f64;
    let mut g: f64;
    let mut b: f64;

    if s > 0.0 {
        if h == 1.0 {
            h = 0.0;
        }

        let i = (h * 6.0) as u32;
        let f = h * 6.0 - i as f64;

        let w = v * (1.0 - s);
        let q = v * (1.0 - s * f);
        let t = v * (1.0 - s * (1.0 - f));

        (r, g, b) = match i {
            0 => (v, t, w),
            1 => (q, v, w),
            2 => (w, v, t),
            3 => (w, q, v),
            4 => (t, w, v),
            5 => (v, w, q),
            _ => (v, v, v),
        }
    } else {
        (r, g, b) = (v, v, v);
    }

    r = r * 255.0;
    g = g * 255.0;
    b = b * 255.0;

    let rgb: RGBColor = [r as u8, g as u8, b as u8];
    rgb
}

pub struct DiscTexture {
    pub left: opengl_graphics::Texture,
    pub middle: opengl_graphics::Texture,
    pub right: opengl_graphics::Texture,
    pub left_highlight: opengl_graphics::Texture,
    pub middle_highlight: opengl_graphics::Texture,
    pub right_highlight: opengl_graphics::Texture,
}

fn load_image(path: PathBuf) -> RgbaImage {
    let img = image::open(path).expect("File not found");
    img.to_rgba8()
}

fn load_images(paths: Vec<PathBuf>) -> Vec<RgbaImage> {
    let mut images: Vec<RgbaImage> = vec![];
    for path in paths {
        images.push(load_image(path));
    }
    images
}

fn apply_color(mut image: RgbaImage, color: RGBColor) -> RgbaImage {
    for pixel in image.pixels_mut() {
        let data = &mut pixel.0;

        for i in 0..3 {
            data[i] = ((data[i] as u32 * color[i] as u32) / 255) as u8;
        }
    }

    image
}

fn apply_color_all(images: Vec<RgbaImage>, color: RGBColor) -> Vec<RgbaImage> {
    let mut processed_images: Vec<RgbaImage> = vec![];

    for image in images {
        processed_images.push(apply_color(image, color));
    }

    processed_images
}

fn get_file_paths(dir: &Path, files: Vec<&str>) -> Vec<PathBuf> {
    let mut file_paths: Vec<PathBuf> = vec![];

    for file in files {
        file_paths.push(dir.join(file));
    }

    file_paths
}

pub fn load_disc_texture_color(color: HSVColor) -> DiscTexture {
    // Resolve file paths
    let dir = Path::new("./assets");
    let files: Vec<&str> = vec![
        "block_grayscale_left.png",
        "block_grayscale_middle.png",
        "block_grayscale_right.png",
        "block_grayscale_highlight_left.png",
        "block_grayscale_highlight_middle.png",
        "block_grayscale_highlight_right.png",
    ];
    let file_paths = get_file_paths(&dir, files);

    // Load images
    let images = load_images(file_paths);

    // // Create textures for higlight
    // let color_highlight: RGBColor = [255, 255, 100];

    let color = hsv_to_rgb(color);
    // let images_highlight = apply_color_all(images.clone(), color_highlight);
    let images = apply_color_all(images, color);

    let left = opengl_graphics::Texture::from_image(
        &images[0],
        &opengl_graphics::TextureSettings::new().mag(opengl_graphics::Filter::Nearest),
    );
    let middle = opengl_graphics::Texture::from_image(
        &images[1],
        &opengl_graphics::TextureSettings::new().mag(opengl_graphics::Filter::Nearest),
    );
    let right = opengl_graphics::Texture::from_image(
        &images[2],
        &opengl_graphics::TextureSettings::new().mag(opengl_graphics::Filter::Nearest),
    );
    let left_highlight = opengl_graphics::Texture::from_image(
        &images[3],
        &opengl_graphics::TextureSettings::new().mag(opengl_graphics::Filter::Nearest),
    );
    let middle_highlight = opengl_graphics::Texture::from_image(
        &images[4],
        &opengl_graphics::TextureSettings::new().mag(opengl_graphics::Filter::Nearest),
    );
    let right_highlight = opengl_graphics::Texture::from_image(
        &images[5],
        &opengl_graphics::TextureSettings::new().mag(opengl_graphics::Filter::Nearest),
    );

    DiscTexture {
        left,
        middle,
        right,
        left_highlight,
        middle_highlight,
        right_highlight,
    }
}

// pub fn load_disc_texture() -> DiscTexture {
//     // let color: Color = [0, 0, 0];
//     // let images = apply_color_all(images, color);

//     let left = opengl_graphics::Texture::from_path(
//         std::path::Path::new("./assets/block2_left.png"),
//         &opengl_graphics::TextureSettings::new().mag(opengl_graphics::Filter::Nearest),
//     )
//     .unwrap();
//     let middle = opengl_graphics::Texture::from_path(
//         std::path::Path::new("./assets/block2_middle.png"),
//         &opengl_graphics::TextureSettings::new().mag(opengl_graphics::Filter::Nearest),
//     )
//     .unwrap();
//     let right = opengl_graphics::Texture::from_path(
//         std::path::Path::new("./assets/block2_right.png"),
//         &opengl_graphics::TextureSettings::new().mag(opengl_graphics::Filter::Nearest),
//     )
//     .unwrap();
//     let left_highlight = opengl_graphics::Texture::from_path(
//         std::path::Path::new("./assets/block2_highlight_left.png"),
//         &opengl_graphics::TextureSettings::new().mag(opengl_graphics::Filter::Nearest),
//     )
//     .unwrap();
//     let middle_highlight = opengl_graphics::Texture::from_path(
//         std::path::Path::new("./assets/block2_highlight_middle.png"),
//         &opengl_graphics::TextureSettings::new().mag(opengl_graphics::Filter::Nearest),
//     )
//     .unwrap();
//     let right_highlight = opengl_graphics::Texture::from_path(
//         std::path::Path::new("./assets/block2_highlight_right.png"),
//         &opengl_graphics::TextureSettings::new().mag(opengl_graphics::Filter::Nearest),
//     )
//     .unwrap();

//     DiscTexture {
//         left,
//         middle,
//         right,
//         left_highlight,
//         middle_highlight,
//         right_highlight,
//     }
// }

pub fn compute_disc_color(disc: u32, n_discs: u32) -> RGBColor {
    let hsv: HSVColor = [((255 / (n_discs)) * disc) as u8, 255, 255];
    hsv
}

pub struct RodTexture {
    pub normal: opengl_graphics::Texture,
    pub highlight: opengl_graphics::Texture,
}

pub fn load_rod_texture() -> RodTexture {
    let image_normal = load_image(PathBuf::from("./assets/rod2.png"));
    let image_higlight = load_image(PathBuf::from("./assets/rod2_highlight.png"));

    let normal = opengl_graphics::Texture::from_image(
        &image_normal,
        &opengl_graphics::TextureSettings::new().mag(opengl_graphics::Filter::Nearest),
    );
    let highlight = opengl_graphics::Texture::from_image(
        &image_higlight,
        &opengl_graphics::TextureSettings::new().mag(opengl_graphics::Filter::Nearest),
    );

    RodTexture { normal, highlight }
}
