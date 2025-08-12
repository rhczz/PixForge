# PixForge

用 Rust 编写的快速高效的图像格式转换命令行工具。

## 特性

- **多格式支持**: 支持 PNG、JPEG、WebP、GIF、ICO 格式之间的转换
- **批量处理**: 转换整个目录中的图像文件
- **智能检测**: 通过文件内容自动检测图像格式

## 安装

### 从源码编译

```bash
git clone https://github.com/your-username/pixforge.git
cd pixforge
cargo build --release
```

## 使用方法

### 基本语法

```bash
pixforge --to <格式> <输入> [选项]
```

### 示例

**转换单个图像:**
```bash
pixforge --to webp image.png
```

**自定义质量转换:**
```bash
pixforge --to jpeg image.png --quality 90
```

**自定义输出目录:**
```bash
pixforge --to png image.jpg --output ./converted/
```

**批量转换目录中的所有图像:**
```bash
pixforge --to webp ./photos/ --output ./converted/
```

**详细输出:**
```bash
pixforge --to png image.jpg --verbose
```

### 选项

| 选项 | 简写 | 描述 | 默认值 |
|------|------|------|--------|
| `--to 格式` | | 目标格式 (png, jpeg, jpg, gif, webp, ico) | 必需 |
| `--output 目录` | `-o` | 输出目录 | 与输入相同 |
| `--quality 质量` | `-q` | 质量 (0-100) | 80 |
| `--verbose` | `-v` | 显示详细转换信息 | false |
| `--help` | `-h` | 显示帮助信息 | |

## 许可证

MIT 许可证 - 详情请查看 LICENSE 文件。