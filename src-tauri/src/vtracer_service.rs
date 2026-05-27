use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use visioncortex::PathSimplifyMode;
use vtracer::{ColorImage, Config, Hierarchical, Preset, SvgFile};

pub const ALLOWED_EXTENSIONS: [&str; 8] = ["png", "jpg", "jpeg", "bmp", "webp", "gif", "tif", "tiff"];

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CropRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertRequest {
    pub input_path: String,
    pub output_path: Option<String>,
    pub settings: VtracerSettings,
    pub crop: Option<CropRect>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewRequest {
    pub input_path: String,
    pub settings: VtracerSettings,
    pub crop: Option<CropRect>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchRequest {
    pub input_paths: Vec<String>,
    pub settings: VtracerSettings,
    pub output_mode: String,
    pub output_dir: Option<String>,
    pub preserve_subdirs: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSvgRequest {
    pub source_path: String,
    pub output_path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertResponse {
    pub input_path: String,
    pub output_path: String,
    pub elapsed_ms: u128,
    pub output_size_bytes: u64,
    pub path_count: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewResponse {
    pub svg_content: String,
    pub width: u32,
    pub height: u32,
    pub path_count: u32,
    pub elapsed_ms: u128,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
    pub file_size_bytes: u64,
    pub mime: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageDataUrlResponse {
    pub data_url: String,
    pub width: u32,
    pub height: u32,
    pub file_size_bytes: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchFailedItem {
    pub path: String,
    pub error: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchResponse {
    pub succeeded: Vec<ConvertResponse>,
    pub failed: Vec<BatchFailedItem>,
    pub total_elapsed_ms: u128,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSvgResponse {
    pub output_path: String,
    pub output_size_bytes: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VtracerSettings {
    pub preset: Option<String>,
    pub colormode: String,
    pub hierarchical: String,
    pub mode: String,
    pub filter_speckle: u32,
    pub color_precision: i32,
    pub gradient_step: i32,
    pub corner_threshold: i32,
    pub segment_length: f64,
    pub splice_threshold: i32,
    pub path_precision: u32,
    pub max_iterations: usize,
    #[serde(default)]
    pub trim_empty_space: bool,
    #[serde(default)]
    pub svgo_optimize: bool,
}

impl Default for VtracerSettings {
    fn default() -> Self {
        Self {
            preset: None,
            colormode: "color".to_string(),
            hierarchical: "stacked".to_string(),
            mode: "spline".to_string(),
            filter_speckle: 4,
            color_precision: 6,
            gradient_step: 16,
            corner_threshold: 60,
            segment_length: 4.0,
            splice_threshold: 45,
            path_precision: 2,
            max_iterations: 10,
            trim_empty_space: true,
            svgo_optimize: false,
        }
    }
}

pub fn convert_to_svg(request: ConvertRequest) -> Result<ConvertResponse, String> {
    let input_path = PathBuf::from(request.input_path.trim());
    validate_input_path(&input_path)?;
    let output_path = normalize_output_path(&input_path, request.output_path.as_deref())?;
    let trim_empty = request.settings.trim_empty_space;
    let svgo_optimize = request.settings.svgo_optimize;
    let config = build_config(request.settings)?;

    let started = Instant::now();
    let (svg, _) = convert_image_at_path(&input_path, config, trim_empty, request.crop.as_ref())?;
    write_svg_file(&svg, &output_path)?;
    if svgo_optimize {
        crate::svg_postprocess::optimize_svg_file(&output_path)?;
    }
    let elapsed_ms = started.elapsed().as_millis();
    let path_count = svg.paths.len() as u32;

    let metadata = std::fs::metadata(&output_path).map_err(|err| {
        format!(
            "Could not read output file metadata `{}`: {err}",
            output_path.display()
        )
    })?;

    Ok(ConvertResponse {
        input_path: input_path.to_string_lossy().to_string(),
        output_path: output_path.to_string_lossy().to_string(),
        elapsed_ms,
        output_size_bytes: metadata.len(),
        path_count,
    })
}

pub fn convert_preview(request: PreviewRequest) -> Result<PreviewResponse, String> {
    let input_path = PathBuf::from(request.input_path.trim());
    validate_input_path(&input_path)?;
    let trim_empty = request.settings.trim_empty_space;
    let config = build_config(request.settings)?;

    let started = Instant::now();
    let (svg, _) = convert_image_at_path(&input_path, config, trim_empty, request.crop.as_ref())?;
    let elapsed_ms = started.elapsed().as_millis();

    Ok(PreviewResponse {
        svg_content: svg.to_string(),
        width: svg.width as u32,
        height: svg.height as u32,
        path_count: svg.paths.len() as u32,
        elapsed_ms,
    })
}

pub fn get_image_info(input_path: String) -> Result<ImageInfo, String> {
    let path = PathBuf::from(input_path.trim());
    validate_input_path(&path)?;
    let img = image::open(&path).map_err(|e| format!("Could not open image: {e}"))?;
    let (width, height) = (img.width(), img.height());
    let metadata = std::fs::metadata(&path).map_err(|e| e.to_string())?;
    let mime = mime_from_extension(&path);
    Ok(ImageInfo {
        width,
        height,
        file_size_bytes: metadata.len(),
        mime,
    })
}

pub fn read_image_data_url(
    input_path: String,
    trim_empty: Option<bool>,
) -> Result<ImageDataUrlResponse, String> {
    use base64::{engine::general_purpose::STANDARD, Engine as _};

    let path = PathBuf::from(input_path.trim());
    validate_input_path(&path)?;
    let trim = trim_empty.unwrap_or(false);
    let img_rgba = read_color_image(&path, trim, None)?;
    let img = image::RgbaImage::from_raw(
        img_rgba.width as u32,
        img_rgba.height as u32,
        img_rgba.pixels,
    )
    .ok_or_else(|| "Could not build image for preview".to_string())?;
    let mut buffer = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(img)
        .write_to(&mut buffer, image::ImageFormat::Png)
        .map_err(|e| format!("Could not encode preview: {e}"))?;
    let bytes = buffer.into_inner();
    let mime = mime_from_extension(&path);
    let data_url = format!("data:{mime};base64,{}", STANDARD.encode(&bytes));
    Ok(ImageDataUrlResponse {
        data_url,
        width: img_rgba.width as u32,
        height: img_rgba.height as u32,
        file_size_bytes: bytes.len() as u64,
    })
}

pub fn scan_images_in_directory(dir_path: String) -> Result<Vec<String>, String> {
    let root = PathBuf::from(dir_path.trim());
    if !root.is_dir() {
        return Err(format!("Path is not a directory: {}", root.display()));
    }
    let mut results = Vec::new();
    collect_images_recursive(&root, &root, &mut results)?;
    results.sort();
    Ok(results)
}

pub fn convert_batch(
    request: BatchRequest,
    cancel: Arc<AtomicBool>,
    on_progress: impl Fn(usize, usize, &str, &str, Option<&str>),
) -> BatchResponse {
    let started = Instant::now();
    let mut succeeded = Vec::new();
    let mut failed = Vec::new();
    let total = request.input_paths.len();
    let config = match build_config(request.settings.clone()) {
        Ok(c) => c,
        Err(e) => {
            for (index, path) in request.input_paths.iter().enumerate() {
                on_progress(index, total, path, "error", Some(&e));
                failed.push(BatchFailedItem {
                    path: path.clone(),
                    error: e.clone(),
                });
            }
            return BatchResponse {
                succeeded,
                failed,
                total_elapsed_ms: started.elapsed().as_millis(),
            };
        }
    };

    let output_mode = request.output_mode.trim();
    let output_dir = request.output_dir.as_ref().map(|p| PathBuf::from(p.trim()));
    let preserve_subdirs = request.preserve_subdirs.unwrap_or(false);

    for (index, raw_path) in request.input_paths.iter().enumerate() {
        if cancel.load(Ordering::SeqCst) {
            on_progress(index, total, raw_path, "error", Some("Cancelled by user"));
            break;
        }

        on_progress(index, total, raw_path, "running", None);

        let input_path = PathBuf::from(raw_path.trim());
        if let Err(err) = validate_input_path(&input_path) {
            on_progress(index, total, raw_path, "error", Some(&err));
            failed.push(BatchFailedItem {
                path: raw_path.clone(),
                error: err,
            });
            continue;
        }

        let output_path = match resolve_batch_output_path(
            &input_path,
            output_mode,
            output_dir.as_deref(),
            preserve_subdirs,
        ) {
            Ok(p) => p,
            Err(err) => {
                on_progress(index, total, raw_path, "error", Some(&err));
                failed.push(BatchFailedItem {
                    path: raw_path.clone(),
                    error: err,
                });
                continue;
            }
        };

        if let Some(parent) = output_path.parent() {
            if let Err(err) = std::fs::create_dir_all(parent) {
                let msg = format!("Could not create directory: {err}");
                on_progress(index, total, raw_path, "error", Some(&msg));
                failed.push(BatchFailedItem {
                    path: raw_path.clone(),
                    error: msg,
                });
                continue;
            }
        }

        let trim_empty = request.settings.trim_empty_space;
        let svgo_optimize = request.settings.svgo_optimize;

        match convert_image_at_path(&input_path, config.clone(), trim_empty, None) {
            Ok((svg, _)) => match write_svg_file(&svg, &output_path) {
                Ok(()) => {
                    if svgo_optimize {
                        if let Err(err) = crate::svg_postprocess::optimize_svg_file(&output_path) {
                            on_progress(index, total, raw_path, "error", Some(&err));
                            failed.push(BatchFailedItem {
                                path: raw_path.clone(),
                                error: err,
                            });
                            continue;
                        }
                    }
                    let path_count = svg.paths.len() as u32;
                    let metadata = std::fs::metadata(&output_path).ok();
                    let response = ConvertResponse {
                        input_path: raw_path.clone(),
                        output_path: output_path.to_string_lossy().to_string(),
                        elapsed_ms: 0,
                        output_size_bytes: metadata.map(|m| m.len()).unwrap_or(0),
                        path_count,
                    };
                    on_progress(index, total, raw_path, "done", None);
                    succeeded.push(response);
                }
                Err(err) => {
                    on_progress(index, total, raw_path, "error", Some(&err));
                    failed.push(BatchFailedItem {
                        path: raw_path.clone(),
                        error: err,
                    });
                }
            },
            Err(err) => {
                on_progress(index, total, raw_path, "error", Some(&err));
                failed.push(BatchFailedItem {
                    path: raw_path.clone(),
                    error: err,
                });
            }
        }
    }

    BatchResponse {
        succeeded,
        failed,
        total_elapsed_ms: started.elapsed().as_millis(),
    }
}

pub fn save_svg_as(request: SaveSvgRequest) -> Result<SaveSvgResponse, String> {
    let source_path = PathBuf::from(request.source_path.trim());
    if !source_path.exists() {
        return Err(format!(
            "SVG source not found: `{}`",
            source_path.to_string_lossy()
        ));
    }

    if source_path.extension().and_then(|ext| ext.to_str()) != Some("svg") {
        return Err("Source must be a .svg file".to_string());
    }

    let output_path = ensure_svg_extension(PathBuf::from(request.output_path.trim()))?;
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| {
            format!(
                "Could not create directory `{}`: {err}",
                parent.to_string_lossy()
            )
        })?;
    }

    std::fs::copy(&source_path, &output_path).map_err(|err| {
        format!(
            "Could not save SVG to `{}`: {err}",
            output_path.to_string_lossy()
        )
    })?;

    let metadata = std::fs::metadata(&output_path).map_err(|err| {
        format!(
            "Could not read saved SVG metadata `{}`: {err}",
            output_path.to_string_lossy()
        )
    })?;

    Ok(SaveSvgResponse {
        output_path: output_path.to_string_lossy().to_string(),
        output_size_bytes: metadata.len(),
    })
}

fn convert_image_at_path(
    input_path: &Path,
    config: Config,
    trim_empty: bool,
    crop: Option<&CropRect>,
) -> Result<(SvgFile, ColorImage), String> {
    let img = read_color_image(input_path, trim_empty, crop)?;
    let svg = vtracer::convert(img.clone(), config)?;
    Ok((svg, img))
}

fn read_color_image(
    input_path: &Path,
    trim_empty: bool,
    crop: Option<&CropRect>,
) -> Result<ColorImage, String> {
    let mut img = image::open(input_path)
        .map_err(|_| format!("Could not read image: {}", input_path.display()))?
        .to_rgba8();

    if trim_empty {
        img = crate::svg_postprocess::trim_transparent_borders(img);
    }

    if let Some(crop_rect) = crop {
        img = apply_crop_rect(img, crop_rect)?;
    }

    let (width, height) = (img.width() as usize, img.height() as usize);
    Ok(ColorImage {
        pixels: img.as_raw().to_vec(),
        width,
        height,
    })
}

fn apply_crop_rect(img: image::RgbaImage, crop: &CropRect) -> Result<image::RgbaImage, String> {
    let (iw, ih) = (img.width(), img.height());
    if crop.width == 0 || crop.height == 0 {
        return Err("Crop area must be larger than zero".to_string());
    }
    if crop.x >= iw || crop.y >= ih {
        return Err("Crop area extends outside the image".to_string());
    }
    let w = crop.width.min(iw - crop.x);
    let h = crop.height.min(ih - crop.y);
    if w < 1 || h < 1 {
        return Err("Crop area is too small".to_string());
    }
    Ok(image::imageops::crop_imm(&img, crop.x, crop.y, w, h).to_image())
}

fn write_svg_file(svg: &SvgFile, output_path: &Path) -> Result<(), String> {
    let mut out_file = File::create(output_path)
        .map_err(|e| format!("Could not create file `{}`: {e}", output_path.display()))?;
    write!(out_file, "{svg}").map_err(|e| format!("Could not write SVG: {e}"))
}

fn collect_images_recursive(
    root: &Path,
    current: &Path,
    results: &mut Vec<String>,
) -> Result<(), String> {
    for entry in std::fs::read_dir(current).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            collect_images_recursive(root, &path, results)?;
        } else if is_supported_path(&path) {
            results.push(path.to_string_lossy().to_string());
        }
    }
    Ok(())
}

fn resolve_batch_output_path(
    input_path: &Path,
    output_mode: &str,
    output_dir: Option<&Path>,
    preserve_subdirs: bool,
) -> Result<PathBuf, String> {
    let file_name = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "Could not determine file name".to_string())?;

    match output_mode {
        "output_dir" => {
            let dir = output_dir.ok_or_else(|| "Output directory not set".to_string())?;
            if preserve_subdirs {
                if let Some(parent) = input_path.parent() {
                    let relative = parent
                        .file_name()
                        .map(|n| dir.join(n))
                        .unwrap_or_else(|| dir.to_path_buf());
                    return Ok(relative.join(format!("{file_name}.svg")));
                }
            }
            Ok(dir.join(format!("{file_name}.svg")))
        }
        _ => {
            let mut out = input_path.to_path_buf();
            out.set_file_name(format!("{file_name}.svg"));
            Ok(out)
        }
    }
}

fn mime_from_extension(path: &Path) -> String {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        Some("bmp") => "image/bmp",
        Some("tif") | Some("tiff") => "image/tiff",
        _ => "application/octet-stream",
    }
    .to_string()
}

fn is_supported_path(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| {
            let lower = e.to_ascii_lowercase();
            ALLOWED_EXTENSIONS.contains(&lower.as_str())
        })
        .unwrap_or(false)
}

pub fn validate_input_path(input_path: &Path) -> Result<(), String> {
    if !input_path.exists() {
        return Err(format!(
            "Input file not found: `{}`",
            input_path.to_string_lossy()
        ));
    }

    if !input_path.is_file() {
        return Err(format!(
            "Input path is not a file: `{}`",
            input_path.to_string_lossy()
        ));
    }

    if !is_supported_path(input_path) {
        let ext = input_path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("?");
        return Err(format!(
            "Unsupported format `{ext}`. Supported: {}",
            ALLOWED_EXTENSIONS.join(", ")
        ));
    }

    Ok(())
}

fn normalize_output_path(input_path: &Path, output_path: Option<&str>) -> Result<PathBuf, String> {
    let candidate = match output_path {
        Some(path) if !path.trim().is_empty() => PathBuf::from(path.trim()),
        _ => {
            let mut generated = input_path.to_path_buf();
            generated.set_extension("svg");
            generated
        }
    };
    ensure_svg_extension(candidate)
}

fn ensure_svg_extension(mut output_path: PathBuf) -> Result<PathBuf, String> {
    if output_path.as_os_str().is_empty() {
        return Err("Empty SVG save path".to_string());
    }

    if output_path.extension().is_none() {
        output_path.set_extension("svg");
    }

    if output_path.extension().and_then(|ext| ext.to_str()) != Some("svg") {
        return Err("Output file must have .svg extension".to_string());
    }

    Ok(output_path)
}

pub fn build_config(settings: VtracerSettings) -> Result<Config, String> {
    if settings.filter_speckle > 16 {
        return Err("filterSpeckle must be in range [0, 16]".to_string());
    }
    if !(1..=8).contains(&settings.color_precision) {
        return Err("colorPrecision must be in range [1, 8]".to_string());
    }
    if !(0..=255).contains(&settings.gradient_step) {
        return Err("gradientStep must be in range [0, 255]".to_string());
    }
    if !(0..=180).contains(&settings.corner_threshold) {
        return Err("cornerThreshold must be in range [0, 180]".to_string());
    }
    if !(3.5..=10.0).contains(&settings.segment_length) {
        return Err("segmentLength must be in range [3.5, 10.0]".to_string());
    }
    if !(0..=180).contains(&settings.splice_threshold) {
        return Err("spliceThreshold must be in range [0, 180]".to_string());
    }
    if settings.path_precision > 12 {
        return Err("pathPrecision must be in range [0, 12]".to_string());
    }
    if settings.max_iterations == 0 || settings.max_iterations > 32 {
        return Err("maxIterations must be in range [1, 32]".to_string());
    }

    let mut config = match settings.preset {
        Some(raw) if !raw.trim().is_empty() => Config::from_preset(
            Preset::from_str(raw.trim())
                .map_err(|err| format!("Invalid preset `{}`: {err}", raw.trim()))?,
        ),
        _ => Config::default(),
    };

    config.color_mode = vtracer::ColorMode::from_str(normalize_colormode(&settings.colormode)?)
        .map_err(|err| format!("Invalid colormode: {err}"))?;
    config.hierarchical = Hierarchical::from_str(settings.hierarchical.trim())
        .map_err(|err| format!("Invalid hierarchical: {err}"))?;
    config.mode = parse_mode(&settings.mode)?;
    config.filter_speckle = settings.filter_speckle as usize;
    config.color_precision = settings.color_precision;
    config.layer_difference = settings.gradient_step;
    config.corner_threshold = settings.corner_threshold;
    config.length_threshold = settings.segment_length;
    config.splice_threshold = settings.splice_threshold;
    config.path_precision = Some(settings.path_precision);
    config.max_iterations = settings.max_iterations;
    Ok(config)
}

fn parse_mode(raw_mode: &str) -> Result<PathSimplifyMode, String> {
    match raw_mode.trim().to_ascii_lowercase().as_str() {
        "pixel" | "none" => Ok(PathSimplifyMode::None),
        "polygon" => Ok(PathSimplifyMode::Polygon),
        "spline" => Ok(PathSimplifyMode::Spline),
        _ => Err(format!(
            "Invalid mode `{raw_mode}`. Allowed: pixel, polygon, spline"
        )),
    }
}

fn normalize_colormode(raw: &str) -> Result<&'static str, String> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "bw" | "binary" => Ok("binary"),
        "color" => Ok("color"),
        _ => Err(format!(
            "Invalid colormode `{raw}`. Allowed: color, bw"
        )),
    }
}
