use anyhow::Result;
use image::ImageFormat;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedImageInfo {
    pub file_path: String,
    pub actual_size: u64,
    pub width: u32,
    pub height: u32,
}

pub struct ContentProcessor {
    imgs_dir: PathBuf,
}

impl ContentProcessor {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir().ok_or_else(|| anyhow::anyhow!("无法获取配置目录"))?;

        let imgs_dir = config_dir.join("clipboard-app").join("imgs");
        std::fs::create_dir_all(&imgs_dir)?;

        Ok(Self { imgs_dir })
    }

    pub async fn process_image_with_dimensions(
        &self,
        image_data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<SavedImageInfo> {
        log::info!(
            "[ContentProcessor] 开始处理指定尺寸图片: {}x{}, 数据大小: {} 字节",
            width,
            height,
            image_data.len()
        );

        // 验证数据长度是否匹配RGBA格式
        let expected_size = (width * height * 4) as usize;
        if image_data.len() == expected_size {
            log::debug!(
                "[ContentProcessor] 数据大小匹配RGBA格式 ({}字节)，直接处理",
                expected_size
            );
            return self
                .process_raw_rgba_data_with_info(image_data, width, height)
                .await;
        }

        // 如果不匹配，可能是其他格式，尝试标准处理
        log::warn!(
            "[ContentProcessor] 数据大小不匹配RGBA ({} != {})，尝试标准图片处理",
            image_data.len(),
            expected_size
        );
        self.process_image_with_info(image_data, width, height)
            .await
    }

    pub async fn process_image(&self, image_data: &[u8]) -> Result<String> {
        log::info!(
            "[ContentProcessor] 开始处理未知尺寸图片，数据大小: {} 字节",
            image_data.len()
        );
        log::trace!(
            "[ContentProcessor] 数据前32字节: {:02X?}",
            &image_data[..image_data.len().min(32)]
        );

        // 首先检查是否是原始像素数据
        if let Some((width, height)) = self.detect_raw_rgba_data(image_data) {
            log::info!(
                "[ContentProcessor] 检测到原始RGBA数据: {}x{}",
                width,
                height
            );
            return self.process_raw_rgba_data(image_data, width, height).await;
        }

        // 如果不是标准分辨率，但数据长度是4的倍数，可能仍然是RGBA数据
        if image_data.len().is_multiple_of(4) && image_data.len() >= 64 {
            log::debug!(
                "[process_image] 数据长度符合RGBA格式 ({})，尝试作为原始像素数据处理",
                image_data.len()
            );
            // 使用简单的正方形或矩形推断
            let pixel_count = image_data.len() / 4;
            let sqrt_pixels = (pixel_count as f64).sqrt() as u32;

            log::debug!(
                "[process_image] 像素数: {}, 平方根: {}",
                pixel_count,
                sqrt_pixels
            );

            // 尝试几种可能的尺寸
            let possible_dimensions = vec![
                (sqrt_pixels, pixel_count as u32 / sqrt_pixels),
                (pixel_count as u32 / sqrt_pixels, sqrt_pixels),
                (sqrt_pixels + 1, pixel_count as u32 / (sqrt_pixels + 1)),
                (pixel_count as u32 / (sqrt_pixels + 1), sqrt_pixels + 1),
            ];

            for (w, h) in possible_dimensions {
                if w > 0 && h > 0 && (w * h) as usize == pixel_count {
                    log::debug!("[process_image] 尝试使用推断尺寸: {}x{}", w, h);
                    match self.process_raw_rgba_data(image_data, w, h).await {
                        Ok(result) => {
                            log::debug!("[process_image] 成功使用尺寸 {}x{}", w, h);
                            return Ok(result);
                        }
                        Err(e) => log::debug!("[process_image] 尺寸 {}x{} 处理失败: {}", w, h, e),
                    }
                }
            }
        }

        // 使用 infer 库进行标准格式检测
        if let Some(mime_type) = infer::get(image_data) {
            if !mime_type.mime_type().starts_with("image/") {
                log::warn!(
                    "[ContentProcessor] 跳过非图片数据，检测到类型: {}",
                    mime_type.mime_type()
                );
                return Err(anyhow::anyhow!(
                    "数据不是图片格式: {}",
                    mime_type.mime_type()
                ));
            }

            log::info!(
                "[ContentProcessor] 检测到标准图片格式: {}",
                mime_type.mime_type()
            );
        } else {
            // 如果 infer 无法检测，再检查是否可能是图片数据
            if !self.is_likely_image_data(image_data) {
                log::warn!("[ContentProcessor] 无法识别的数据格式，可能不是图片");
                return Err(anyhow::anyhow!("无法识别的数据格式"));
            }
            log::debug!("[ContentProcessor] infer无法识别格式，但数据可能是图片，继续尝试处理");
        }

        // 生成唯一文件名
        let filename = format!("{}.png", Uuid::new_v4());
        let file_path = self.imgs_dir.join(&filename);

        log::debug!("[process_image] 准备保存图片到: {:?}", file_path);

        // 尝试使用多种方式解析并保存图片
        let img = match image::load_from_memory(image_data) {
            Ok(img) => {
                log::info!(
                    "[ContentProcessor] 成功使用自动格式检测加载图片: {}x{}",
                    img.width(),
                    img.height()
                );
                img
            }
            Err(e) => {
                log::debug!("[ContentProcessor] 自动格式检测失败: {}", e);
                // 如果无法自动检测格式，尝试指定格式
                let formats = [
                    ImageFormat::Png,
                    ImageFormat::Jpeg,
                    ImageFormat::Gif,
                    ImageFormat::Bmp,
                    ImageFormat::Tiff,
                    ImageFormat::WebP,
                ];

                let mut _last_error = None;
                for format in formats.iter() {
                    match image::load_from_memory_with_format(image_data, *format) {
                        Ok(img) => {
                            log::info!(
                                "[ContentProcessor] 成功使用指定格式 {:?} 加载图片: {}x{}",
                                format,
                                img.width(),
                                img.height()
                            );
                            return self.save_image(img, &file_path).await;
                        }
                        Err(e) => {
                            log::trace!("[ContentProcessor] 格式 {:?} 加载失败: {}", format, e);
                            _last_error = Some(e);
                        }
                    }
                }

                // 如果所有格式都失败，但确实是图片数据，保存原始数据
                log::warn!("[ContentProcessor] 所有标准格式解码失败，尝试保存原始数据");
                return self.save_raw_image_data(image_data, &file_path).await;
            }
        };

        self.save_image(img, &file_path).await
    }

    async fn process_image_with_info(
        &self,
        image_data: &[u8],
        _expected_width: u32,
        _expected_height: u32,
    ) -> Result<SavedImageInfo> {
        log::debug!(
            "[process_image_with_info] 开始处理图片数据，大小: {} 字节",
            image_data.len()
        );

        // 生成唯一文件名
        let filename = format!("{}.png", Uuid::new_v4());
        let file_path = self.imgs_dir.join(&filename);

        // 使用 infer 库进行标准格式检测
        if let Some(mime_type) = infer::get(image_data) {
            if !mime_type.mime_type().starts_with("image/") {
                return Err(anyhow::anyhow!(
                    "数据不是图片格式: {}",
                    mime_type.mime_type()
                ));
            }
        }

        // 尝试解析并保存图片
        let img = image::load_from_memory(image_data)?;
        let (actual_width, actual_height) = (img.width(), img.height());

        self.save_image(img, &file_path).await?;

        // 获取实际保存的文件大小
        let actual_size = std::fs::metadata(&file_path)?.len();

        log::info!(
            "[process_image_with_info] 成功处理图片: {}x{}, 压缩后大小: {} 字节",
            actual_width,
            actual_height,
            actual_size
        );

        Ok(SavedImageInfo {
            file_path: format!("imgs/{}", filename),
            actual_size,
            width: actual_width,
            height: actual_height,
        })
    }

    async fn save_image(
        &self,
        img: image::DynamicImage,
        file_path: &std::path::Path,
    ) -> Result<String> {
        // 压缩图片：保持分辨率但优化质量和文件大小
        let compressed_img = self.compress_image(img)?;
        compressed_img.save(file_path)?;

        // 记录压缩后的实际文件大小
        if let Ok(metadata) = std::fs::metadata(file_path) {
            let compressed_size = metadata.len();
            log::debug!("[save_image] 压缩后文件大小: {} 字节", compressed_size);
        }

        let filename = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("无法获取文件名"))?;
        Ok(format!("imgs/{}", filename))
    }

    fn compress_image(&self, img: image::DynamicImage) -> Result<image::DynamicImage> {
        use image::DynamicImage;

        // 获取原始尺寸
        let (width, height) = (img.width(), img.height());
        log::debug!("[compress_image] 原始图片尺寸: {}x{}", width, height);

        // 如果图片太大（超过4K），缩小到合理尺寸但保持宽高比
        let max_dimension = 3840; // 4K 最大边长
        let img = if width > max_dimension || height > max_dimension {
            let ratio = (max_dimension as f32) / (width.max(height) as f32);
            let new_width = (width as f32 * ratio) as u32;
            let new_height = (height as f32 * ratio) as u32;
            log::debug!("[compress_image] 缩放图片到: {}x{}", new_width, new_height);
            img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3)
        } else {
            img
        };

        // 检查是否有透明通道
        let has_alpha = matches!(
            img,
            DynamicImage::ImageRgba8(_) | DynamicImage::ImageRgba16(_)
        );

        if has_alpha {
            // PNG格式保留透明度，这里直接返回调整后的图片
            // PNG格式会在save时自动压缩
            log::debug!("[compress_image] 保留PNG格式（含透明通道）");
            Ok(img)
        } else {
            // 对于不含透明通道的图片，仍然保存为PNG但可以进行更激进的优化
            // 因为后续保存时会自动选择合适的格式
            log::debug!("[compress_image] 无透明通道，将进行优化");
            Ok(img)
        }
    }

    async fn process_raw_rgba_data(
        &self,
        rgba_data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<String> {
        log::debug!(
            "[process_raw_rgba_data] 开始处理RGBA数据: {}x{}, 数据大小: {} 字节",
            width,
            height,
            rgba_data.len()
        );

        // 生成唯一文件名
        let filename = format!("{}.png", Uuid::new_v4());
        let file_path = self.imgs_dir.join(&filename);

        log::debug!("[process_raw_rgba_data] 准备保存到: {:?}", file_path);

        // macOS 剪贴板可能提供BGRA格式而不是RGBA，需要转换
        let mut converted_data = rgba_data.to_vec();

        // 尝试检测是否需要BGRA到RGBA的转换
        // 通过检查alpha通道是否合理来判断（大部分alpha值应该是255或0）
        let mut needs_bgra_conversion = false;
        let sample_size = (rgba_data.len() / 4).min(100); // 采样前100个像素
        let mut alpha_values = Vec::new();
        for i in 0..sample_size {
            alpha_values.push(rgba_data[i * 4 + 3]);
        }

        // 如果大部分alpha值都不是255或0，可能是BGRA格式
        let valid_alpha_count = alpha_values.iter().filter(|&&a| a == 255 || a == 0).count();
        if valid_alpha_count < sample_size / 2 {
            log::debug!("[process_raw_rgba_data] 检测到可能是BGRA格式，尝试转换");
            // 交换B和R通道
            for i in 0..(converted_data.len() / 4) {
                let b = converted_data[i * 4];
                let r = converted_data[i * 4 + 2];
                converted_data[i * 4] = r;
                converted_data[i * 4 + 2] = b;
            }
            needs_bgra_conversion = true;
        }

        // 尝试创建图像缓冲区
        let mut img_buffer = image::ImageBuffer::from_raw(width, height, converted_data.clone());

        // 如果第一次尝试失败，可能是尺寸错误，尝试转置
        if img_buffer.is_none() && height != width {
            log::debug!(
                "[process_raw_rgba_data] 尝试转置尺寸: {}x{} -> {}x{}",
                width,
                height,
                height,
                width
            );
            img_buffer = image::ImageBuffer::from_raw(height, width, converted_data.clone());
        }

        // 如果还是失败，并且没有尝试过BGRA转换，尝试原始数据
        if img_buffer.is_none() && needs_bgra_conversion {
            log::debug!("[process_raw_rgba_data] BGRA转换失败，尝试原始数据");
            img_buffer = image::ImageBuffer::from_raw(width, height, rgba_data.to_vec());
        }

        let img_buffer = img_buffer.ok_or_else(|| {
            log::error!(
                "[process_raw_rgba_data] 无法创建图像缓冲区，尺寸: {}x{}, 数据长度: {}",
                width,
                height,
                rgba_data.len()
            );
            anyhow::anyhow!("无法从原始数据创建图像缓冲区")
        })?;

        let dynamic_img = image::DynamicImage::ImageRgba8(img_buffer);

        // 压缩后保存
        let compressed_img = self.compress_image(dynamic_img)?;
        compressed_img.save(&file_path)?;

        let filename = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("无法获取文件名"))?;

        log::info!(
            "[process_raw_rgba_data] 成功处理原始数据并保存为: {}",
            filename
        );
        Ok(format!("imgs/{}", filename))
    }

    async fn process_raw_rgba_data_with_info(
        &self,
        rgba_data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<SavedImageInfo> {
        log::debug!(
            "[process_raw_rgba_data_with_info] 开始处理RGBA数据: {}x{}, 数据大小: {} 字节",
            width,
            height,
            rgba_data.len()
        );

        // 生成唯一文件名
        let filename = format!("{}.png", Uuid::new_v4());
        let file_path = self.imgs_dir.join(&filename);

        // 处理RGBA数据
        let mut converted_data = rgba_data.to_vec();

        // BGRA到RGBA转换逻辑（与原来相同）
        let mut needs_bgra_conversion = false;
        let sample_size = (rgba_data.len() / 4).min(100);
        let mut alpha_values = Vec::new();
        for i in 0..sample_size {
            alpha_values.push(rgba_data[i * 4 + 3]);
        }

        let valid_alpha_count = alpha_values.iter().filter(|&&a| a == 255 || a == 0).count();
        if valid_alpha_count < sample_size / 2 {
            log::debug!("[process_raw_rgba_data_with_info] 检测到可能是BGRA格式，尝试转换");
            for i in 0..(converted_data.len() / 4) {
                let b = converted_data[i * 4];
                let r = converted_data[i * 4 + 2];
                converted_data[i * 4] = r;
                converted_data[i * 4 + 2] = b;
            }
            needs_bgra_conversion = true;
        }

        let img_buffer = image::ImageBuffer::from_raw(width, height, converted_data.clone())
            .or_else(|| {
                if needs_bgra_conversion {
                    image::ImageBuffer::from_raw(width, height, rgba_data.to_vec())
                } else {
                    None
                }
            })
            .ok_or_else(|| anyhow::anyhow!("无法从原始数据创建图像缓冲区"))?;

        let dynamic_img = image::DynamicImage::ImageRgba8(img_buffer);
        let compressed_img = self.compress_image(dynamic_img)?;
        compressed_img.save(&file_path)?;

        // 获取实际保存的文件大小
        let actual_size = std::fs::metadata(&file_path)?.len();

        log::info!(
            "[process_raw_rgba_data_with_info] 成功处理原始数据: {}x{}, 压缩后大小: {} 字节",
            width,
            height,
            actual_size
        );

        Ok(SavedImageInfo {
            file_path: format!("imgs/{}", filename),
            actual_size,
            width,
            height,
        })
    }

    async fn save_raw_image_data(
        &self,
        image_data: &[u8],
        file_path: &std::path::Path,
    ) -> Result<String> {
        // 尝试根据检测到的格式使用正确的扩展名
        let (extension, actual_path) = if let Some(mime_type) = infer::get(image_data) {
            let ext = match mime_type.mime_type() {
                "image/png" => "png",
                "image/jpeg" => "jpg",
                "image/gif" => "gif",
                "image/webp" => "webp",
                "image/bmp" => "bmp",
                "image/tiff" => "tiff",
                _ => "bin", // 未知格式用 .bin
            };
            let new_path = file_path.with_extension(ext);
            (ext.to_string(), new_path)
        } else {
            ("bin".to_string(), file_path.to_path_buf())
        };

        // 将原始数据直接保存为文件
        std::fs::write(&actual_path, image_data)?;
        let filename = actual_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("无法获取文件名"))?;

        log::info!("保存原始图片数据: {} ({})", filename, extension);
        Ok(format!("imgs/{}", filename))
    }

    fn detect_raw_rgba_data(&self, data: &[u8]) -> Option<(u32, u32)> {
        // 基本检查：必须是4的倍数且有足够的数据
        if data.len() < 16 || !data.len().is_multiple_of(4) {
            return None;
        }

        log::debug!(
            "[detect_raw_rgba_data] 检测RGBA数据: 总长度 {} 字节, {} 个像素",
            data.len(),
            data.len() / 4
        );

        // 简化检测逻辑：如果数据长度是4的倍数，就假设是RGBA数据
        // 现代截图工具通常提供的就是原始像素数据
        let pixel_count = data.len() / 4;

        // 推断尺寸的策略：
        // 1. 先尝试常见分辨率
        let common_dimensions = [
            // macOS Retina 显示器常见分辨率
            (2880, 1800),
            (2560, 1600),
            (2304, 1440),
            (2048, 1280),
            (1920, 1200),
            (1680, 1050),
            (1440, 900),
            (1280, 800),
            // 标准显示器分辨率
            (1920, 1080),
            (2560, 1440),
            (3840, 2160),
            (1366, 768),
            (1280, 720),
            (1600, 900),
            (800, 600),
            (1024, 768),
            (640, 480),
            // 常见截图区域尺寸（包括各种窗口大小）
            (1024, 600),
            (800, 480),
            (600, 400),
            (400, 300),
            (300, 200),
            (200, 150),
            (150, 100),
            (100, 100),
            // 移动设备常见尺寸
            (375, 812),
            (414, 896),
            (390, 844),
            (428, 926),
            // 正方形或接近正方形
            (512, 512),
            (1024, 1024),
            (256, 256),
            (128, 128),
            (64, 64),
            (32, 32),
            (16, 16),
        ];

        for &(w, h) in &common_dimensions {
            if (w * h) as usize == pixel_count {
                log::debug!("[detect_raw_rgba_data] 匹配到常见分辨率: {}x{}", w, h);
                return Some((w, h));
            }
        }

        // 1.5 尝试常见宽高比的变体（考虑部分截图的情况）
        for &(base_w, base_h) in &common_dimensions {
            // 尝试不同的缩放因子
            for scale in [2, 3, 4, 5, 6, 7, 8, 10, 12, 15, 20] {
                let w = base_w / scale;
                let h = base_h / scale;
                if w > 0 && h > 0 && (w * h) as usize == pixel_count {
                    log::debug!(
                        "[detect_raw_rgba_data] 匹配到缩放分辨率: {}x{} (基于 {}x{}/{})",
                        w,
                        h,
                        base_w,
                        base_h,
                        scale
                    );
                    return Some((w, h));
                }

                let w = base_w * scale;
                let h = base_h * scale;
                if (w * h) as usize == pixel_count {
                    log::debug!(
                        "[detect_raw_rgba_data] 匹配到放大分辨率: {}x{} (基于 {}x{}*{})",
                        w,
                        h,
                        base_w,
                        base_h,
                        scale
                    );
                    return Some((w, h));
                }
            }
        }

        // 2. 尝试找到合理的因数分解
        let sqrt_pixels = (pixel_count as f64).sqrt();
        let sqrt_int = sqrt_pixels as u32;

        // 检查是否是完全平方数
        if (sqrt_int * sqrt_int) as usize == pixel_count {
            log::debug!(
                "[detect_raw_rgba_data] 检测到正方形图像: {}x{}",
                sqrt_int,
                sqrt_int
            );
            return Some((sqrt_int, sqrt_int));
        }

        // 3. 尝试常见宽高比，从sqrt附近开始搜索
        let search_range = 100; // 增加搜索范围
        for width in (sqrt_int.saturating_sub(search_range))..=(sqrt_int + search_range) {
            if width == 0 {
                continue;
            }
            if pixel_count.is_multiple_of(width as usize) {
                let height = (pixel_count / width as usize) as u32;
                let ratio = width as f64 / height as f64;

                // 检查是否是合理的宽高比 (0.1 到 10.0) - 放宽限制以支持更多截图
                if (0.1..=10.0).contains(&ratio) {
                    // 优先选择接近常见宽高比的尺寸
                    let common_ratios = [
                        16.0 / 9.0,
                        4.0 / 3.0,
                        3.0 / 2.0,
                        1.0,
                        2.0 / 3.0,
                        3.0 / 4.0,
                        9.0 / 16.0,
                    ];
                    for &target_ratio in &common_ratios {
                        if (ratio - target_ratio).abs() < 0.05 {
                            log::debug!("[detect_raw_rgba_data] 找到接近标准比例的尺寸: {}x{} (比例: {:.2}, 接近 {:.2})", 
                                    width, height, ratio, target_ratio);
                            return Some((width, height));
                        }
                    }
                    // 如果不接近标准比例但仍然合理，也接受
                    if width >= 10 && height >= 10 {
                        log::debug!(
                            "[detect_raw_rgba_data] 找到合理尺寸: {}x{} (比例: {:.2})",
                            width,
                            height,
                            ratio
                        );
                        return Some((width, height));
                    }
                }
            }
        }

        // 4. 尝试更广泛的因数分解（限制在合理范围内）
        let max_width = (pixel_count as f64 * 2.0).sqrt() as u32;
        let min_width = ((pixel_count as f64).sqrt() / 2.0) as u32;

        for width in min_width..=max_width {
            if width == 0 {
                continue;
            }
            if pixel_count.is_multiple_of(width as usize) {
                let height = (pixel_count / width as usize) as u32;
                if height >= 10 && width >= 10 {
                    // 确保尺寸不会太小
                    let ratio = width as f64 / height as f64;
                    if (0.05..=20.0).contains(&ratio) {
                        // 非常宽松的比例限制
                        log::debug!(
                            "[detect_raw_rgba_data] 使用扩展因数分解得到尺寸: {}x{} (比例: {:.2})",
                            width,
                            height,
                            ratio
                        );
                        return Some((width, height));
                    }
                }
            }
        }

        // 5. 绝对最后的尝试：如果数据看起来可能是图片，尝试一些极端情况
        if pixel_count > 100 {
            // 至少100个像素
            // 尝试作为单行或单列图像
            if pixel_count < 1000000 {
                // 限制在合理范围内
                log::debug!("[detect_raw_rgba_data] 尝试作为单行图像: {}x1", pixel_count);
                return Some((pixel_count as u32, 1));
            }
        }

        log::debug!(
            "[detect_raw_rgba_data] 无法推断图像尺寸，像素数: {}",
            pixel_count
        );
        None
    }

    fn is_likely_image_data(&self, data: &[u8]) -> bool {
        if data.len() < 2 {
            return false;
        }

        // 检查PNG
        if data.len() >= 4 && data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            return true;
        }

        // 检查JPEG (多种变体)
        if data.len() >= 3 && data.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return true;
        }

        // 检查GIF
        if data.len() >= 4 && data.starts_with(&[0x47, 0x49, 0x46, 0x38]) {
            return true;
        }

        // 检查BMP
        if data.len() >= 2 && data.starts_with(&[0x42, 0x4D]) {
            return true;
        }

        // 检查TIFF
        if data.len() >= 4
            && (
                data.starts_with(&[0x49, 0x49, 0x2A, 0x00]) || // little-endian
            data.starts_with(&[0x4D, 0x4D, 0x00, 0x2A])
                // big-endian
            )
        {
            return true;
        }

        // 检查WebP (RIFF container + WEBP signature)
        if data.len() >= 12 && data.starts_with(&[0x52, 0x49, 0x46, 0x46]) {
            // 检查是否是WebP格式: "WEBP" signature at bytes 8-11
            if &data[8..12] == b"WEBP" {
                return true;
            }
        }

        false
    }
}
