use eframe::egui;
use std::path::Path;
use std::fs;
use std::env;
use walkdir::WalkDir;
use image::{DynamicImage,GenericImageView, ImageBuffer, Rgba};

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size((320.0, 240.0)),
        ..Default::default()
    };
    let result = eframe::run_native(
        "图片切割工具",
        options,
        Box::new(|cc| {
            // 获取系统字体路径
            let system_root = env::var("SystemRoot").expect("无法获取系统根目录");
            let font_path = format!("{}\\Fonts\\msyh.ttc", system_root); // 字体文件路径

            // 加载字体文件
            let font_data = fs::read(font_path).expect("无法加载字体文件");

            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "Microsoft YaHei".to_owned(),          // 字体名称
                egui::FontData::from_owned(font_data), // 加载字体数据
            );
            fonts
                .families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, "Microsoft YaHei".to_owned()); // 使用系统字体
            cc.egui_ctx.set_fonts(fonts.clone()); // 使用 clone 避免移动所有权

            Box::new(ImageCutterApp::default())
        }),
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

fn split_image(img: DynamicImage) -> Vec<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let (width, height) = img.dimensions();

    // 计算水平和垂直方向需要切割的份数
    let horizontal_splits = (width as f32 / 512.0).ceil() as u32;
    let vertical_splits = (height as f32 / 512.0).ceil() as u32;

    // 计算每份的宽度和高度
    let chunk_width = (width as f32 / horizontal_splits as f32).ceil() as u32;
    let chunk_height = (height as f32 / vertical_splits as f32).ceil() as u32;

    let mut chunks = Vec::new();

    for y in 0..vertical_splits {
        for x in 0..horizontal_splits {
            let start_x = x * chunk_width;
            let start_y = y * chunk_height;

            // 确保最后一块不会超出图片边界
            let end_x = std::cmp::min(start_x + chunk_width, width);
            let end_y = std::cmp::min(start_y + chunk_height, height);

            let chunk = img.crop_imm(start_x, start_y, end_x - start_x, end_y - start_y);
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