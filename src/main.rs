use anyhow::Result;
use clap::Parser;
use std::path::{Path, PathBuf};

mod converter;
mod utils;

use converter::ImageConverter;

/// 支持的图像格式列表
const SUPPORTED_FORMATS: &[&str] = &["png", "jpeg", "jpg", "gif", "webp", "ico"];

/// PixForge - 强大的图像格式转换工具
///
/// 支持批量转换和单文件转换，提供质量控制选项
#[derive(Parser)]
#[command(name = "pixforge")]
#[command(about = "🎨 PixForge - 优雅的图像格式转换工具")]
#[command(version = "0.1.0")]
#[command(author = "PixForge Team")]
struct CliArgs {
    /// 目标格式 (png, jpeg, jpg, gif, webp, ico)
    #[arg(long, value_name = "FORMAT")]
    #[arg(help = "目标图像格式")]
    to: String,

    /// 输入文件或目录路径
    #[arg(value_name = "INPUT")]
    #[arg(help = "要转换的图像文件或包含图像的目录")]
    input: PathBuf,
    
    /// 输出目录 (默认与输入文件同目录)
    #[arg(short = 'o', long, value_name = "OUTPUT")]
    #[arg(help = "输出目录，默认与输入文件在同一目录")]
    output: Option<PathBuf>,

    /// 图像质量 (0-100，默认80)
    #[arg(short = 'q', long, value_name = "QUALITY")]
    #[arg(help = "图像质量控制，范围0-100，80是推荐值")]
    #[arg(value_parser = clap::value_parser!(u8).range(0..=100))]
    quality: Option<u8>,

    /// 详细输出模式
    #[arg(short = 'v', long)]
    #[arg(help = "显示详细的转换信息")]
    verbose: bool,
}

fn main() -> Result<()> {
    // 正常模式：解析命令行参数
    let args = CliArgs::parse();

    // 验证输入路径
    validate_input_path(&args.input)?;

    // 验证目标格式
    validate_target_format(&args.to)?;

    // 确定输出路径
    let output_path = determine_output_path(&args.input, &args.output);

    // 获取质量设置
    let quality = args.quality.unwrap_or(80);
    
    if args.verbose {
        print_conversion_info(&args.input, &output_path, &args.to, quality);
    }

    // 执行转换
    let converter = ImageConverter::new();

    if args.input.is_file() {
        println!("🖼️  单文件转换模式");
        converter.convert_single_file(&args.input, &output_path, &args.to, quality)?;
    } else {
        println!("📁 批量转换模式");
        converter.convert_directory(&args.input, &output_path, &args.to, quality)?;
    }

    Ok(())
}

/// 验证输入路径是否存在
fn validate_input_path(input: &Path) -> Result<()> {
    if !input.exists() {
        anyhow::bail!("❌ 输入路径不存在: {}", input.display());
    }
    Ok(())
}

/// 验证目标格式是否支持
fn validate_target_format(format: &str) -> Result<()> {
    if !SUPPORTED_FORMATS.contains(&format.to_lowercase().as_str()) {
        anyhow::bail!(
            "❌ 不支持的目标格式: {}\n📋 支持的格式: {}",
            format,
            SUPPORTED_FORMATS.join(", ")
        );
    }
    Ok(())
}

/// 确定输出路径
fn determine_output_path(input: &Path, output: &Option<PathBuf>) -> PathBuf {
    match output {
        Some(path) => path.clone(),
        None => {
            if input.is_file() {
                // 单文件：输出到同目录
                input.parent()
                    .unwrap_or_else(|| Path::new("."))
                    .to_path_buf()
            } else {
                // 目录：创建pixforge子目录
                input.parent()
                    .unwrap_or_else(|| Path::new("."))
                    .join("pixforge_output")
            }
        }
    }
}

/// 打印转换信息
fn print_conversion_info(input: &Path, output: &Path, format: &str, quality: u8) {
    println!("🔧 转换配置:");
    println!("   📂 输入: {}", input.display());
    println!("   📁 输出: {}", output.display());
    println!("   🎯 格式: {}", format.to_uppercase());
    println!("   ⚡ 质量: {}%", quality);
    println!();
}
