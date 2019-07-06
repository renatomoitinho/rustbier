use crate::commons::errors::*;
use crate::commons::*;
use opencv::core;
use opencv::imgcodecs;
use opencv::imgproc;
use opencv::prelude::*;
use opencv::types::*;

pub fn process_image(
    buffer: &[u8],
    request: ProcessImageRequest,
    png_quality: u8,
) -> Result<Vec<u8>, opencv::Error> {
    info!("Processing request {:?}", request);
    let mat_buf = core::Mat::from_slice(buffer)?;
    let src_mat = imgcodecs::imdecode(&mat_buf, imgcodecs::IMREAD_COLOR)?;
    let resized = resize_image(&src_mat, &request.size)?;
    let enc_quality = match request.format {
        ImageFormat::Png => png_quality as i32,
        _ => request.quality,
    };
    let quality = get_encode_params(&request.format, enc_quality as i32);
    let mut rs_buf = VectorOfuchar::new();
    let watermarked = if let Some(watermark) = request.watermark {
        apply_watermark(&resized, &watermark)?
    } else {
        resized
    };

    debug!("Encoding to: {}", request.format);
    imgcodecs::imencode(
        format!(".{}", request.format).as_str(),
        &watermarked,
        &mut rs_buf,
        &quality,
    )?;
    Ok(rs_buf.to_vec())
}

fn get_encode_params(f: &ImageFormat, q: i32) -> VectorOfint {
    let mut quality = VectorOfint::with_capacity(2);
    match f {
        ImageFormat::Jpeg => {
            quality.push(imgcodecs::IMWRITE_JPEG_QUALITY);
            quality.push(q);
        }
        ImageFormat::Png => {
            quality.push(imgcodecs::IMWRITE_PNG_COMPRESSION);
            quality.push(q);
        }
        ImageFormat::Webp => {
            quality.push(imgcodecs::IMWRITE_WEBP_QUALITY);
            quality.push(q);
        }
    };
    quality
}

fn apply_watermark(img: &core::Mat, watermark: &Watermark) -> Result<core::Mat, opencv::Error> {
    let mat_buf = core::Mat::from_slice(watermark.file)?;
    let wm_mat = imgcodecs::imdecode(&mat_buf, imgcodecs::IMREAD_COLOR)?;
    let resized_wm = resize_image(&wm_mat, &watermark.size)?;

    let (left, top, right, bottom) = get_watermark_borders(
        img.cols()?,
        img.rows()?,
        resized_wm.cols()?,
        resized_wm.rows()?,
        &watermark.position,
        &watermark.origin,
    );

    debug!("Adding borders to make images the same size. Padding: top: {}, left: {}, bottom: {}, right: {}", top, left, bottom, right);

    let mut wm_expanded = core::Mat::new()?;
    core::copy_make_border(
        &resized_wm,
        &mut wm_expanded,
        top,
        bottom,
        left,
        right,
        core::BORDER_CONSTANT,
        core::Scalar::new(0.0f64, 0.0f64, 0.0f64, 1.0f64),
    )?;

    debug!("Image size: {}x{}", img.cols()?, img.rows()?);
    debug!(
        "Watermark size: {}x{}",
        wm_expanded.cols()?,
        wm_expanded.rows()?
    );

    let mut result_mat = core::Mat::new()?;
    core::add_weighted(
        &img,
        1.0f64,
        &wm_expanded,
        watermark.alpha,
        0.0f64,
        &mut result_mat,
        img.depth()?,
    )?;
    Ok(result_mat)
}

fn resize_image(img: &core::Mat, size: &Size) -> Result<core::Mat, opencv::Error> {
    let original_width = img.cols()?;
    let original_height = img.rows()?;

    debug!(
        "Resizing image. Original size: {}x{}. Desired: {:?}",
        original_width, original_height, size
    );

    let (target_width, target_height) = get_target_size(original_width, original_height, &size)?;

    debug!("Final size: {}x{}", target_width, target_height);
    let mut result = core::Mat::new()?;

    imgproc::resize(
        img,
        &mut result,
        core::Size {
            width: target_width,
            height: target_height,
        },
        0f64,
        0f64,
        imgproc::INTER_LINEAR,
    )?;

    Ok(result)
}

fn get_ratio(desired_measure: i32, original_measure: i32, opposite_orig_measure: i32) -> i32 {
    let ratio = desired_measure as f32 / original_measure as f32;
    (opposite_orig_measure as f32 * ratio) as i32
}

fn is_negative_or_zero(size: &Size) -> bool {
    (size.height.is_some() && size.height.unwrap() <= 0)
        || (size.width.is_some() && size.width.unwrap() <= 0)
}

fn get_target_size(
    original_width: i32,
    original_height: i32,
    desired_size: &Size,
) -> Result<(i32, i32), InvalidSizeError> {
    match &desired_size {
        Size {
            width: None,
            height: None,
        } => Ok((original_width, original_height)),
        s if is_negative_or_zero(s) => Err(InvalidSizeError::new(&desired_size)),
        Size {
            width: Some(w),
            height: Some(h),
        } if *h > original_height && *w > original_width => Ok((original_width, original_height)),
        Size {
            width: Some(w),
            height: Some(h),
        } => {
            let diff_height = *h as f32 / original_height as f32;
            let diff_width = *w as f32 / original_width as f32;

            if diff_height < diff_width && diff_height <= 1.0 {
                Ok((get_ratio(*h, original_height, original_width), *h))
            } else {
                Ok((*w, get_ratio(*w, original_width, original_height)))
            }
        }
        Size {
            width: None,
            height: Some(h),
        } => {
            if *h > original_height {
                Ok((original_width, original_height))
            } else {
                Ok((get_ratio(*h, original_height, original_width), *h))
            }
        }
        Size {
            width: Some(w),
            height: None,
        } => {
            if *w > original_width {
                Ok((original_width, original_height))
            } else {
                Ok((*w, get_ratio(*w, original_width, original_height)))
            }
        }
    }
}

fn get_watermark_borders(
    width: i32,
    height: i32,
    wm_width: i32,
    wm_height: i32,
    point: &Point,
    origin: &WatermarkPosition,
) -> (i32, i32, i32, i32) {
    match origin {
        WatermarkPosition::Center => {
            let left = (width / 2) - (wm_width / 2);
            let top = (height / 2) - (wm_height / 2);
            let odd_w_acc = width % 2;
            let odd_h_acc = height % 2;
            (left, top, left + odd_w_acc, top + odd_h_acc)
        }
        WatermarkPosition::LeftTop => {
            let right = width - point.x - wm_width;
            let bottom = height - point.y - wm_height;
            let left = point.x + if right < 0 { right } else { 0 };
            let top = point.y + if bottom < 0 { bottom } else { 0 };
            (
                left,
                top,
                if right > 0 { right } else { 0 },
                if bottom > 0 { bottom } else { 0 },
            )
        }
        WatermarkPosition::RightBottom => {
            let left = width - point.x - wm_width;
            let top = height - point.y - wm_height;
            let right = point.x + if left < 0 { left } else { 0 };
            let bottom = point.y + if top < 0 { top } else { 0 };
            (
                if left > 0 { left } else { 0 },
                if top > 0 { top } else { 0 },
                right,
                bottom,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_size() {
        assert!(get_target_size(
            100,
            100,
            &Size {
                width: Some(-1),
                height: Some(-1)
            }
        )
        .is_err());
        assert!(get_target_size(
            100,
            100,
            &Size {
                width: Some(-1),
                height: Some(1)
            }
        )
        .is_err());
        assert!(get_target_size(
            100,
            100,
            &Size {
                width: Some(1),
                height: Some(-1)
            }
        )
        .is_err());
        assert!(get_target_size(
            100,
            100,
            &Size {
                width: None,
                height: Some(-1)
            }
        )
        .is_err());
        assert!(get_target_size(
            100,
            100,
            &Size {
                width: Some(-1),
                height: None
            }
        )
        .is_err());
    }

    #[test]
    fn test_size_square_img() {
        assert_eq!(
            get_target_size(
                100,
                100,
                &Size {
                    width: Some(100),
                    height: Some(100)
                }
            ),
            Ok((100, 100))
        );
        assert_eq!(
            get_target_size(
                100,
                100,
                &Size {
                    width: Some(10),
                    height: Some(10)
                }
            ),
            Ok((10, 10))
        );
        assert_eq!(
            get_target_size(
                100,
                100,
                &Size {
                    width: Some(10),
                    height: Some(20)
                }
            ),
            Ok((10, 10))
        );
        assert_eq!(
            get_target_size(
                100,
                100,
                &Size {
                    width: Some(20),
                    height: Some(10)
                }
            ),
            Ok((10, 10))
        );
        assert_eq!(
            get_target_size(
                100,
                100,
                &Size {
                    width: Some(100),
                    height: Some(50)
                }
            ),
            Ok((50, 50))
        );
        assert_eq!(
            get_target_size(
                100,
                100,
                &Size {
                    width: Some(50),
                    height: Some(100)
                }
            ),
            Ok((50, 50))
        );
        assert_eq!(
            get_target_size(
                100,
                100,
                &Size {
                    width: Some(120),
                    height: Some(100)
                }
            ),
            Ok((100, 100))
        );
        assert_eq!(
            get_target_size(
                100,
                100,
                &Size {
                    width: Some(100),
                    height: Some(120)
                }
            ),
            Ok((100, 100))
        );
    }

    #[test]
    fn test_size_rectangular_img() {
        assert_eq!(
            get_target_size(
                100,
                150,
                &Size {
                    width: Some(100),
                    height: Some(150)
                }
            ),
            Ok((100, 150))
        );
        assert_eq!(
            get_target_size(
                100,
                150,
                &Size {
                    width: Some(100),
                    height: Some(100)
                }
            ),
            Ok((66, 100))
        );
        assert_eq!(
            get_target_size(
                100,
                150,
                &Size {
                    width: Some(120),
                    height: Some(100)
                }
            ),
            Ok((66, 100))
        );
        assert_eq!(
            get_target_size(
                100,
                150,
                &Size {
                    width: Some(100),
                    height: Some(50)
                }
            ),
            Ok((33, 50))
        );
        assert_eq!(
            get_target_size(
                100,
                150,
                &Size {
                    width: Some(50),
                    height: Some(100)
                }
            ),
            Ok((50, 75))
        );
        assert_eq!(
            get_target_size(
                100,
                150,
                &Size {
                    width: Some(200),
                    height: Some(200)
                }
            ),
            Ok((100, 150))
        );
        assert_eq!(
            get_target_size(
                100,
                150,
                &Size {
                    width: Some(200),
                    height: Some(150)
                }
            ),
            Ok((100, 150))
        );
        assert_eq!(
            get_target_size(
                100,
                150,
                &Size {
                    width: Some(100),
                    height: Some(200)
                }
            ),
            Ok((100, 150))
        );
    }

    #[test]
    fn test_size_rectangular_img2() {
        assert_eq!(
            get_target_size(
                150,
                100,
                &Size {
                    width: Some(150),
                    height: Some(100)
                }
            ),
            Ok((150, 100))
        );
        assert_eq!(
            get_target_size(
                150,
                100,
                &Size {
                    width: Some(100),
                    height: Some(100)
                }
            ),
            Ok((100, 66))
        );
        assert_eq!(
            get_target_size(
                150,
                100,
                &Size {
                    width: Some(120),
                    height: Some(100)
                }
            ),
            Ok((120, 80))
        );
        assert_eq!(
            get_target_size(
                150,
                100,
                &Size {
                    width: Some(100),
                    height: Some(50)
                }
            ),
            Ok((75, 50))
        );
        assert_eq!(
            get_target_size(
                150,
                100,
                &Size {
                    width: Some(50),
                    height: Some(100)
                }
            ),
            Ok((50, 33))
        );
        assert_eq!(
            get_target_size(
                150,
                100,
                &Size {
                    width: Some(200),
                    height: Some(200)
                }
            ),
            Ok((150, 100))
        );
        assert_eq!(
            get_target_size(
                150,
                100,
                &Size {
                    width: Some(200),
                    height: Some(150)
                }
            ),
            Ok((150, 100))
        );
        assert_eq!(
            get_target_size(
                150,
                100,
                &Size {
                    width: Some(100),
                    height: Some(200)
                }
            ),
            Ok((100, 66))
        );
    }

    #[test]
    fn test_size_optional() {
        assert_eq!(
            get_target_size(
                100,
                100,
                &Size {
                    width: Some(100),
                    height: None
                }
            ),
            Ok((100, 100))
        );
        assert_eq!(
            get_target_size(
                100,
                100,
                &Size {
                    width: None,
                    height: Some(100)
                }
            ),
            Ok((100, 100))
        );
        assert_eq!(
            get_target_size(
                50,
                100,
                &Size {
                    width: Some(100),
                    height: None
                }
            ),
            Ok((50, 100))
        );
        assert_eq!(
            get_target_size(
                100,
                50,
                &Size {
                    width: None,
                    height: Some(100)
                }
            ),
            Ok((100, 50))
        );
        assert_eq!(
            get_target_size(
                150,
                100,
                &Size {
                    width: Some(100),
                    height: None
                }
            ),
            Ok((100, 66))
        );
        assert_eq!(
            get_target_size(
                100,
                150,
                &Size {
                    width: None,
                    height: Some(100)
                }
            ),
            Ok((66, 100))
        );
        assert_eq!(
            get_target_size(
                100,
                100,
                &Size {
                    width: None,
                    height: None
                }
            ),
            Ok((100, 100))
        );
    }

    #[test]
    fn test_center_watermark() {
        assert_eq!(
            get_watermark_borders(
                100,
                100,
                10,
                10,
                &Point { x: 10, y: 10 },
                &WatermarkPosition::Center
            ),
            (45, 45, 45, 45)
        );
        assert_eq!(
            get_watermark_borders(
                101,
                101,
                10,
                10,
                &Point { x: 10, y: 10 },
                &WatermarkPosition::Center
            ),
            (45, 45, 46, 46)
        );
    }

    #[test]
    fn test_left_top_watermark() {
        assert_eq!(
            get_watermark_borders(
                100,
                100,
                10,
                10,
                &Point { x: 10, y: 10 },
                &WatermarkPosition::LeftTop
            ),
            (10, 10, 80, 80)
        );
        assert_eq!(
            get_watermark_borders(
                100,
                100,
                10,
                10,
                &Point { x: 95, y: 10 },
                &WatermarkPosition::LeftTop
            ),
            (90, 10, 0, 80)
        );
        assert_eq!(
            get_watermark_borders(
                100,
                100,
                10,
                10,
                &Point { x: 10, y: 95 },
                &WatermarkPosition::LeftTop
            ),
            (10, 90, 80, 0)
        );
        assert_eq!(
            get_watermark_borders(
                100,
                100,
                10,
                10,
                &Point { x: 95, y: 95 },
                &WatermarkPosition::LeftTop
            ),
            (90, 90, 0, 0)
        );
    }

    #[test]
    fn test_right_bottom_watermark() {
        assert_eq!(
            get_watermark_borders(
                100,
                100,
                10,
                10,
                &Point { x: 10, y: 10 },
                &WatermarkPosition::RightBottom
            ),
            (80, 80, 10, 10)
        );
        assert_eq!(
            get_watermark_borders(
                100,
                100,
                10,
                10,
                &Point { x: 95, y: 10 },
                &WatermarkPosition::RightBottom
            ),
            (0, 80, 90, 10)
        );
        assert_eq!(
            get_watermark_borders(
                100,
                100,
                10,
                10,
                &Point { x: 10, y: 95 },
                &WatermarkPosition::RightBottom
            ),
            (80, 0, 10, 90)
        );
        assert_eq!(
            get_watermark_borders(
                100,
                100,
                10,
                10,
                &Point { x: 95, y: 95 },
                &WatermarkPosition::RightBottom
            ),
            (0, 0, 90, 90)
        );
    }
}
