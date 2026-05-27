use image::{imageops, RgbaImage};
use std::path::{Path, PathBuf};
use std::process::Command;

const ALPHA_TRIM_THRESHOLD: u8 = 8;

/// Trims transparent margins on RGBA (common issue with oversized PNG canvases).
pub fn trim_transparent_borders(img: RgbaImage) -> RgbaImage {
    let (w, h) = (img.width(), img.height());
    if w == 0 || h == 0 {
        return img;
    }

    let mut min_x = w;
    let mut min_y = h;
    let mut max_x = 0u32;
    let mut max_y = 0u32;
    let mut found = false;

    for y in 0..h {
        for x in 0..w {
            if img.get_pixel(x, y)[3] > ALPHA_TRIM_THRESHOLD {
                found = true;
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }

    if !found {
        return img;
    }

    let crop_w = max_x - min_x + 1;
    let crop_h = max_y - min_y + 1;
    if crop_w == w && crop_h == h {
        return img;
    }

    imageops::crop_imm(&img, min_x, min_y, crop_w, crop_h).to_image()
}

pub fn optimize_svg_file(path: &Path) -> Result<(), String> {
    if run_svgo_cli(path) {
        return Ok(());
    }
    basic_minify_svg_file(path)
}

fn run_svgo_cli(path: &Path) -> bool {
    let path_str = match path.to_str() {
        Some(s) => s,
        None => return false,
    };

    if Command::new("svgo")
        .args(["--multipass", path_str, "-o", path_str])
        .status()
        .is_ok_and(|s| s.success())
    {
        return true;
    }

    if Command::new("npx")
        .args(["--yes", "svgo", "--multipass", path_str, "-o", path_str])
        .status()
        .is_ok_and(|s| s.success())
    {
        return true;
    }

    for local in local_svgo_bins() {
        if Command::new(&local)
            .args(["--multipass", path_str, "-o", path_str])
            .status()
            .is_ok_and(|s| s.success())
        {
            return true;
        }
    }

    false
}

fn local_svgo_bins() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        if let Some(parent) = PathBuf::from(manifest).parent() {
            let local = parent.join("node_modules/.bin/svgo");
            if local.exists() {
                out.push(local);
            }
        }
    }
    out
}

fn basic_minify_svg_file(path: &Path) -> Result<(), String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Could not read SVG `{}`: {e}", path.display()))?;
    let minified = basic_minify_svg(&content);
    std::fs::write(path, minified)
        .map_err(|e| format!("Could not write SVG `{}`: {e}", path.display()))
}

fn basic_minify_svg(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut last_was_space = false;
    for ch in input.chars() {
        if ch.is_whitespace() {
            if !last_was_space {
                out.push(' ');
                last_was_space = true;
            }
        } else {
            out.push(ch);
            last_was_space = false;
        }
    }
    out.trim().to_string()
}
