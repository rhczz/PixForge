use std::fs::File;
use std::io::Read;
use std::path::Path;

/// 支持的图像文件扩展名列表
const SUPPORTED_IMAGE_EXTENSIONS: &[&str] = &[
    "png", "jpeg", "jpg", "gif", "webp", "svg", "ico",
    "bmp", "tiff", "tif", "avif", "heic", "heif"
];

/// 图像文件魔数签名
struct ImageSignature {
    signature: &'static [u8],
    format: &'static str,
}

const IMAGE_SIGNATURES: &[ImageSignature] = &[
    ImageSignature {
        signature: &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
        format: "png",
    },
    ImageSignature {
        signature: &[0xFF, 0xD8, 0xFF],
        format: "jpeg",
    },
    ImageSignature {
        signature: &[0x47, 0x49, 0x46, 0x38],
        format: "gif",
    },
    ImageSignature {
        signature: &[0x52, 0x49, 0x46, 0x46],
        format: "webp", // 需要额外检查WEBP标识
    },
    ImageSignature {
        signature: &[0x00, 0x00, 0x01, 0x00],
        format: "ico",
    },
    ImageSignature {
        signature: &[0x42, 0x4D],
        format: "bmp",
    },
    ImageSignature {
        signature: &[0x49, 0x49, 0x2A, 0x00],
        format: "tiff",
    },
    ImageSignature {
        signature: &[0x4D, 0x4D, 0x00, 0x2A],
        format: "tiff",
    },
];

/// 检查文件是否为图像文件
///
/// 首先检查扩展名，然后验证文件内容的魔数签名
pub fn is_image_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    // 快速扩展名检查
    if !has_potential_image_extension(path) {
        return false;
    }

    // 内容验证
    detect_image_format_by_content(path).is_some()
}

/// 检查文件扩展名是否可能是图像格式
fn has_potential_image_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext_str| {
            SUPPORTED_IMAGE_EXTENSIONS.contains(&ext_str.to_lowercase().as_str())
        })
        .unwrap_or(true) // 没有扩展名也可能是图片文件
}

/// 通过文件内容检测图像格式
///
/// 读取文件头部字节，匹配已知的图像格式魔数签名
pub fn detect_image_format_by_content(path: &Path) -> Option<String> {
    let mut file = File::open(path).ok()?;
    let mut buffer = [0u8; 16];

    let bytes_read = file.read(&mut buffer).ok()?;
    if bytes_read < 4 {
        return None; // 文件太小，不可能是有效图像
    }

    // 检查标准图像格式签名
    for signature in IMAGE_SIGNATURES {
        if buffer.starts_with(signature.signature) {
            // WebP需要额外验证
            if signature.format == "webp" {
                return validate_webp_signature(&buffer);
            }

            // GIF需要额外验证版本号
            if signature.format == "gif" {
                return validate_gif_signature(&buffer);
            }

            return Some(signature.format.to_string());
        }
    }

    // 检查SVG (文本格式)
    if let Ok(text) = std::str::from_utf8(&buffer) {
        if is_svg_content(text) {
            return Some("svg".to_string());
        }
    }

    None
}

/// 验证WebP文件签名
fn validate_webp_signature(buffer: &[u8]) -> Option<String> {
    if buffer.len() >= 12 && &buffer[8..12] == b"WEBP" {
        Some("webp".to_string())
    } else {
        None
    }
}

/// 验证GIF文件签名和版本
fn validate_gif_signature(buffer: &[u8]) -> Option<String> {
    if buffer.len() >= 6
        && &buffer[0..4] == b"GIF8"
        && (buffer[4] == b'7' || buffer[4] == b'9')
        && buffer[5] == b'a' {
        Some("gif".to_string())
    } else {
        None
    }
}

/// 检查是否为SVG内容
fn is_svg_content(text: &str) -> bool {
    let lowercased = text.to_lowercase();
    lowercased.starts_with("<?xml") || lowercased.starts_with("<svg")
}

/// 获取文件扩展名
///
/// 返回小写的扩展名字符串，如果没有扩展名则返回空字符串
pub fn get_extension(path: &Path) -> String {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default()
}

/// 更改文件扩展名
///
/// 保留原文件名的主体部分，替换为新的扩展名
///
/// # 参数
/// * `path` - 原文件路径
/// * `new_extension` - 新扩展名（不包含点号）
///
/// # 返回
/// 新的文件名字符串
pub fn change_extension(path: &Path, new_extension: &str) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem_str| format!("{}.{}", stem_str, new_extension))
        .unwrap_or_else(|| format!("output.{}", new_extension))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_has_potential_image_extension() {
        assert!(has_potential_image_extension(&PathBuf::from("test.png")));
        assert!(has_potential_image_extension(&PathBuf::from("test.jpg")));
        assert!(has_potential_image_extension(&PathBuf::from("test.JPEG")));
        assert!(!has_potential_image_extension(&PathBuf::from("test.txt")));
        assert!(has_potential_image_extension(&PathBuf::from("test"))); // 无扩展名
    }

    #[test]
    fn test_change_extension() {
        let path = PathBuf::from("image.png");
        assert_eq!(change_extension(&path, "webp"), "image.webp");

        let path = PathBuf::from("path/to/image.jpeg");
        assert_eq!(change_extension(&path, "png"), "image.png");
    }

    #[test]
    fn test_get_extension() {
        assert_eq!(get_extension(&PathBuf::from("test.PNG")), "png");
        assert_eq!(get_extension(&PathBuf::from("test.jpeg")), "jpeg");
        assert_eq!(get_extension(&PathBuf::from("test")), "");
    }

    #[test]
    fn test_is_svg_content() {
        assert!(is_svg_content("<?xml version=\"1.0\"?>"));
        assert!(is_svg_content("<svg xmlns=\"http://www.w3.org/2000/svg\">"));
        assert!(is_svg_content("<?XML version=\"1.0\"?>")); // 大小写不敏感
        assert!(!is_svg_content("<html>"));
    }
}
