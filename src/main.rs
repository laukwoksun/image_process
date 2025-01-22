use eframe::egui;
use std::path::Path;
use walkdir::WalkDir;
use image::{GenericImageView, ImageBuffer, Rgba};

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size((320.0, 240.0)),
        ..Default::default()
    };
    let result = eframe::run_native(
        "图片切割工具",
        options,
        Box::new(|_cc| Box::new(ImageCutterApp::default())),
    );
    match result {
        Ok(_) => println!("应用程序正常退出"),
        Err(e) => eprintln!("应用程序启动失败: {}", e),
    }
}

#[derive(Default)]
struct ImageCutterApp {
    folder_path: String,
    message: String,
}

impl eframe::App for ImageCutterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("请输入文件夹路径：");
            ui.text_edit_singleline(&mut self.folder_path);

            if ui.button("确定").clicked() {
                let folder_path = self.folder_path.replace("\\", "/");
                let (total_png_count, processed_png_count) = process_folder(&folder_path);
                self.message = format!(
                    "全部的 PNG 格式文件数量: {}\n最终需要处理的 PNG 文件数量: {}",
                    total_png_count, processed_png_count
                );
            }

            if!self.message.is_empty() {
                ui.label(&self.message);
            }
        });
    }
}

fn process_folder(folder_path: &str) -> (usize, usize) {
    let mut total_png_count = 0;
    let mut processed_png_count = 0;

    for entry in WalkDir::new(folder_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("png") {
            total_png_count += 1;
            if process_png(path) {
                processed_png_count += 1;
            }
        }
    }

    (total_png_count, processed_png_count)
}

fn process_png(path: &Path) -> bool {
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
                return true;
            }
        }
        Err(e) => {
            eprintln!("Failed to open image {}: {}", path.display(), e);
        }
    }
    false
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