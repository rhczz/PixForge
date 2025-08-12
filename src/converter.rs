use anyhow::{Context, Result};
use image::{codecs::jpeg::JpegEncoder, codecs::png::PngEncoder, ColorType, ImageFormat};
use std::fs::{self, File};
use std::path::Path;
use walkdir::WalkDir;

use crate::utils;

/// 图像转换器，提供各种格式间的转换功能
pub struct ImageConverter;

enum ImageType {
    SimpleGraphics, // 简单图形
    HorizontalGraphics, // 水平渐变
    VerticalPattern, // 垂直模糊
    SmoothPhoto, // 平滑照片
    ComplexGeometry, // 复杂几何
    Mixed // 混合内容，使用自适应过滤器
}

impl ImageConverter {
    /// 创建新的图像转换器实例
    pub fn new() -> Self {
        Self
    }
    
    /// 转换单个文件
    /// 
    /// # 参数
    /// * `input` - 输入文件路径
    /// * `output` - 输出路径（可以是文件或目录）
    /// * `target_format` - 目标格式
    /// * `quality` - 质量参数 (0-100)
    pub fn convert_single_file(
        &self, 
        input: &Path, 
        output: &Path, 
        target_format: &str, 
        quality: u8
    ) -> Result<()> {
        if !utils::is_image_file(input) {
            anyhow::bail!("不支持的图像格式: {}", input.display());
        }
        
        let output_file = self.determine_output_path(input, output, target_format);
        
        // 确保输出目录存在
        self.ensure_output_directory(&output_file)?;
        
        self.convert_image(input, &output_file, target_format, quality)
            .with_context(|| format!("转换失败: {}", input.display()))?;
        
        println!("✅ 转换完成: {} -> {}", input.display(), output_file.display());
        Ok(())
    }
    
    /// 批量转换目录中的图片
    /// 
    /// # 参数
    /// * `input_dir` - 输入目录
    /// * `output_dir` - 输出目录
    /// * `target_format` - 目标格式
    /// * `quality` - 质量参数 (0-100)
    pub fn convert_directory(
        &self, 
        input_dir: &Path, 
        output_dir: &Path, 
        target_format: &str, 
        quality: u8
    ) -> Result<()> {
        let mut stats = ConversionStats::new();
        
        // 确保输出目录存在
        fs::create_dir_all(output_dir)?;
        
        println!("🔄 开始批量转换...");
        
        for entry in WalkDir::new(input_dir).into_iter().filter_map(Result::ok) {
            let path = entry.path();
            
            if path.is_file() && utils::is_image_file(path) {
                let relative_path = path.strip_prefix(input_dir)?;
                let output_file = output_dir.join(utils::change_extension(relative_path, target_format));
                
                // 确保输出子目录存在
                self.ensure_output_directory(&output_file)?;
                
                match self.convert_image(path, &output_file, target_format, quality) {
                    Ok(_) => {
                        println!("✅ 转换: {} -> {}", path.display(), output_file.display());
                        stats.increment_converted();
                    }
                    Err(e) => {
                        println!("⚠️  跳过: {} ({})", path.display(), e);
                        stats.increment_skipped();
                    }
                }
            }
        }
        
        stats.print_summary();
        Ok(())
    }
    
    /// 核心图像转换逻辑
    fn convert_image(
        &self, 
        input: &Path, 
        output: &Path, 
        target_format: &str, 
        quality: u8
    ) -> Result<()> {
        // SVG特殊处理
        if utils::get_extension(input).to_lowercase() == "svg" {
            anyhow::bail!("当前版本不支持SVG转换");
        }
        
        let img = image::open(input)
            .with_context(|| format!("无法打开图像: {}", input.display()))?;
        
        let (width, height) = (img.width(), img.height());
        let color_type = img.color();
        let image_type = self.analyze_image_type(&img);

        match target_format.to_lowercase().as_str() {
            "jpeg" | "jpg" => self.convert_to_jpeg(&img, output, quality)?,
            "webp" => self.convert_to_webp(&img, output, quality)?,
            "png" => self.convert_to_png(&img, output, quality, image_type, color_type)?,
            "gif" => self.convert_to_gif(&img, output)?,
            "ico" => self.convert_to_ico(&img, output, width, height)?,
            _ => anyhow::bail!("不支持的目标格式: {}", target_format),
        }
        
        Ok(())
    }

    /// 分析图像类型
    fn analyze_image_type(&self, img: &image::DynamicImage) -> ImageType {
        let (width, height) = (img.width(), img.height());

        // 小尺寸图像通常是图标或者简单图形
        if width <= 64 && height <= 64 {
            return ImageType::SimpleGraphics;
        }

        // 分析图像变化模式
        let sample_size = (width.min(height) / 4).max(10) as usize;
        let rgba_img = img.to_rgba8();

        let mut horizontal_variation = 0u64;
        let mut vertical_variation = 0u64;
        let mut sample_count = 0u32;

        // 采样分析水平和垂直方向的变化
        for y in (0..height).step_by((height as usize / sample_size).max(1)) {
            for x in (1..width).step_by((width as usize / sample_size).max(1)) {
                if let Some(current) = rgba_img.get_pixel_checked(x, y) {
                    if let Some(left) = rgba_img.get_pixel_checked(x - 1, y) {
                        horizontal_variation += Self::pixel_difference(current, left) as u64;
                        sample_count += 1;
                    }
                }
            }
        }

        for y in (1..height).step_by((height as usize / sample_size).max(1)) {
            for x in (0..width).step_by((width as usize / sample_size).max(1)) {
                if let Some(current) = rgba_img.get_pixel_checked(x, y) {
                    if let Some(up) = rgba_img.get_pixel_checked(x, y - 1) {
                        vertical_variation += Self::pixel_difference(current, up) as u64;
                    }
                }
            }
        }

        if sample_count == 0 {
            return ImageType::SimpleGraphics;
        }

        let avg_horizontal = horizontal_variation / sample_count as u64;
        let avg_vertical = vertical_variation / sample_count as u64;

        // 根据方向性变化选择类型
        if avg_horizontal < avg_vertical / 2 {
            ImageType::HorizontalGraphics
        } else if avg_vertical < avg_horizontal / 2 {
            ImageType::VerticalPattern
        } else if avg_horizontal < 10 && avg_vertical < 10 {
            ImageType::SmoothPhoto
        } else if avg_horizontal > 50 && avg_vertical > 50 {
            // 高变化率的复杂内容使用自适应过滤器
            ImageType::Mixed
        } else {
            ImageType::ComplexGeometry
        }
    }

    /// 计算两个像素间的差异
    fn pixel_difference(p1: &image::Rgba<u8>, p2: &image::Rgba<u8>) -> u32 {
        let r_diff = (p1[0] as i32 - p2[0] as i32).abs() as u32;
        let g_diff = (p1[1] as i32 - p2[1] as i32).abs() as u32;
        let b_diff = (p1[2] as i32 - p2[2] as i32).abs() as u32;
        let a_diff = (p1[3] as i32 - p2[3] as i32).abs() as u32;
        r_diff + g_diff + b_diff + a_diff
    }
    
    /// 转换为JPEG格式
    fn convert_to_jpeg(&self, img: &image::DynamicImage, output: &Path, quality: u8) -> Result<()> {
        let output_file = File::create(output)
            .with_context(|| format!("无法创建输出文件: {}", output.display()))?;
        
        let encoder = JpegEncoder::new_with_quality(output_file, quality);
        let rgb_img = img.to_rgb8(); // JPEG不支持透明度
        
        rgb_img.write_with_encoder(encoder)
            .with_context(|| format!("JPEG编码失败: {}", output.display()))?;
        
        Ok(())
    }
    
    /// 转换为WebP格式（使用webp 0.3.0）
    fn convert_to_webp(&self, img: &image::DynamicImage, output: &Path, quality: u8) -> Result<()> {
        let rgba_img = img.to_rgba8();
        let (width, height) = (rgba_img.width(), rgba_img.height());
        
        // 使用webp crate进行编码
        let encoder = webp::Encoder::from_rgba(&rgba_img, width, height);
        let encoded_data = encoder.encode(quality as f32);
        
        fs::write(output, &*encoded_data)
            .with_context(|| format!("WebP保存失败: {}", output.display()))?;
        
        Ok(())
    }
    
    /// 转换为PNG格式
    fn convert_to_png(
        &self, 
        img: &image::DynamicImage, 
        output: &Path, 
        quality: u8,
        image_type: ImageType,
        color_type: ColorType
    ) -> Result<()> {
        let output_file = File::create(output)
            .with_context(|| format!("无法创建输出文件: {}", output.display()))?;

        let filter_type = self.get_optimal_filter_type(image_type);

        // 根据质量参数调整PNG压缩级别
        let compression_level = self.get_png_compression_level(quality);
        let encoder = PngEncoder::new_with_quality(
            output_file, 
            compression_level,
            filter_type
        );

        // 保持原有颜色类型以避免不必要的转换
        self.encode_png_with_optimal_color_type(img, encoder, color_type)?;
        
        Ok(())
    }

    /// 根据图像类型选择最优过滤器
    fn get_optimal_filter_type(&self, image_type: ImageType) -> image::codecs::png::FilterType {
        use image::codecs::png::FilterType;

        match image_type {
            ImageType::SimpleGraphics => FilterType::NoFilter,
            ImageType::HorizontalGraphics => FilterType::Sub,
            ImageType::VerticalPattern => FilterType::Up,
            ImageType::SmoothPhoto => FilterType::Avg,
            ImageType::ComplexGeometry => FilterType::Paeth,
            ImageType::Mixed => FilterType::Adaptive
        }
    }
    
    /// 转换为GIF格式
    fn convert_to_gif(&self, img: &image::DynamicImage, output: &Path) -> Result<()> {
        img.save_with_format(output, ImageFormat::Gif)
            .with_context(|| format!("GIF保存失败: {}", output.display()))?;
        
        Ok(())
    }
    
    /// 转换为ICO格式
    fn convert_to_ico(
        &self, 
        img: &image::DynamicImage, 
        output: &Path, 
        width: u32, 
        height: u32
    ) -> Result<()> {
        // ICO格式尺寸限制处理
        if width > 256 || height > 256 {
            let resized = img.resize(256, 256, image::imageops::FilterType::Lanczos3);
            resized.save_with_format(output, ImageFormat::Ico)
        } else {
            img.save_with_format(output, ImageFormat::Ico)
        }
        .with_context(|| format!("ICO保存失败: {}", output.display()))?;
        
        Ok(())
    }
    
    /// 确定输出文件路径
    fn determine_output_path(&self, input: &Path, output: &Path, target_format: &str) -> std::path::PathBuf {
        if output.is_dir() {
            let filename = utils::change_extension(input, target_format);
            output.join(filename)
        } else {
            output.to_path_buf()
        }
    }
    
    /// 确保输出目录存在
    fn ensure_output_directory(&self, output_file: &Path) -> Result<()> {
        if let Some(parent) = output_file.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(())
    }
    
    /// 获取PNG压缩级别
    fn get_png_compression_level(&self, quality: u8) -> image::codecs::png::CompressionType {
        match quality {
            0..=20 => image::codecs::png::CompressionType::Fast,
            21..=60 => image::codecs::png::CompressionType::Default,
            61..=80 => image::codecs::png::CompressionType::Best,
            _ => image::codecs::png::CompressionType::Best,
        }
    }
    
    /// 使用最优颜色类型编码PNG
    fn encode_png_with_optimal_color_type(
        &self,
        img: &image::DynamicImage,
        encoder: PngEncoder<File>,
        color_type: ColorType
    ) -> Result<()> {
        match color_type {
            ColorType::L8 => {
                let luma_img = img.as_luma8()
                    .map(|img| img.clone())
                    .unwrap_or_else(|| img.to_luma8());
                luma_img.write_with_encoder(encoder)?;
            },
            ColorType::La8 => {
                let luma_alpha_img = img.as_luma_alpha8()
                    .map(|img| img.clone())
                    .unwrap_or_else(|| img.to_luma_alpha8());
                luma_alpha_img.write_with_encoder(encoder)?;
            },
            ColorType::Rgb8 => {
                let rgb_img = img.as_rgb8()
                    .map(|img| img.clone())
                    .unwrap_or_else(|| img.to_rgb8());
                rgb_img.write_with_encoder(encoder)?;
            },
            ColorType::Rgba8 => {
                let rgba_img = img.as_rgba8()
                    .map(|img| img.clone())
                    .unwrap_or_else(|| img.to_rgba8());
                rgba_img.write_with_encoder(encoder)?;
            },
            _ => {
                let rgba_img = img.to_rgba8();
                rgba_img.write_with_encoder(encoder)?;
            }
        }
        Ok(())
    }
}

/// 转换统计信息
#[derive(Debug)]
struct ConversionStats {
    converted: u32,
    skipped: u32,
}

impl ConversionStats {
    fn new() -> Self {
        Self {
            converted: 0,
            skipped: 0,
        }
    }
    
    fn increment_converted(&mut self) {
        self.converted += 1;
    }
    
    fn increment_skipped(&mut self) {
        self.skipped += 1;
    }
    
    fn print_summary(&self) {
        if self.converted == 0 && self.skipped > 0 {
            println!("❌ 没有图片被转换。全部 {} 个文件被跳过。", self.skipped);
        } else {
            println!("🎉 转换完成: {} 个转换成功, {} 个跳过", self.converted, self.skipped);
        }
    }
}
