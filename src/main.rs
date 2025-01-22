use std::path::Path;
use walkdir::WalkDir;
use image::{GenericImageView, ImageBuffer, Rgba};

fn main() {
    let folder_path = "E:\klzz\courses\L8_v1_d_01"; // 替换为实际的文件夹路径
    process_folder(folder_path);
}

fn process_folder(folder_path: &str) {
    for entry in WalkDir::new(folder_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("png") {
            process_png(path);
        }
    }
}

fn process_png(path: &Path) {
    match image::open(path) {
        Ok(img) => {
            let (width, height) = img.dimensions();
            if width > 512 || height > 512 {
                let chunks = split_image(img);
                save_chunks(&chunks, path);
                // 删除原文件
                if let Err(e) = std::fs::remove_file(path) {
                    eprintln!("Failed to delete file {}: {}", path.display(), e);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to open image {}: {}", path.display(), e);
        }
    }
}

fn split_image(img: image::DynamicImage) -> Vec<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let (width, height) = img.dimensions();
    let mut chunks = Vec::new();

    for y in (0..height).step_by(512) {
        for x in (0..width).step_by(512) {
            let chunk_width = std::cmp::min(512, width - x);
            let chunk_height = std::cmp::min(512, height - y);
            let chunk = img.crop_imm(x, y, chunk_width, chunk_height);
            chunks.push(chunk.to_rgba8());
        }
    }

    chunks
}

fn save_chunks(chunks: &[ImageBuffer<Rgba<u8>, Vec<u8>>], original_path: &Path) {
    let file_stem = original_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let parent_dir = original_path.parent().unwrap_or(Path::new(""));

    for (i, chunk) in chunks.iter().enumerate() {
        let new_path = parent_dir.join(format!("{}_{}.png", file_stem, i));
        if let Err(e) = chunk.save(&new_path) {
            eprintln!("Failed to save chunk {}: {}", new_path.display(), e);
        }
    }
}