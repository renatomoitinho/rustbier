use crate::commons::errors::*;
use crate::commons::*;
use opencv::core;
use opencv::imgcodecs;
use opencv::imgproc;
use opencv::prelude::*;
use opencv::types::*;

use magick_rust::bindings::{
    CompositeOperator_OverCompositeOp, FilterType_PointFilter, MagickTransparentPaintImage,
};
use magick_rust::{MagickWand, PixelWand};

pub fn process_image(
    buffer: &[u8],
    request: ProcessImageRequest,
    png_quality: u8,
) -> Result<Vec<u8>, opencv::Error> {
    info!("Processing request {:?}", request);
    let mat_buf = core::Mat::from_slice(buffer)?;
    let src_mat = imgcodecs::imdecode(&mat_buf, imgcodecs::IMREAD_UNCHANGED)?;
    let resized = resize_image(&src_mat, &request.size)?;
    let enc_quality = match request.format {
        ImageFormat::Png => png_quality as i32,
        _ => request.quality,
    };

    let image = if let Some(rotation) = request.rotation {
        rotate_image(&resized, &rotation)?
    } else {
        resized
    };

    let quality = get_encode_params(&request.format, enc_quality as i32);
    let mut rs_buf = VectorOfuchar::new();

    debug!("Encoding to: {}", request.format);
    imgcodecs::imencode(
        format!(".{}", request.format).as_str(),
        &image,
        &mut rs_buf,
        &quality,
    )?;
    if let Some(watermark) = request.watermark {
        apply_watermark(&rs_buf.to_vec(), &watermark, &request.format).map_err(|e| e.into())
    } else {
        Ok(rs_buf.to_vec())
    }
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

fn apply_watermark(
    img: &Vec<u8>,
    watermark: &Watermark,
    format: &ImageFormat,
) -> Result<Vec<u8>, MagickError> {
    let wand = MagickWand::new();
    wand.read_image_blob(img)?;
    let wand_wm = MagickWand::new();
    wand_wm.read_image_blob(watermark.file)?;
    let wm_width = wand_wm.get_image_width() as i32;
    let wm_height = wand_wm.get_image_height() as i32;
    let (wm_target_width, wm_target_height) =
        get_target_size(wm_width, wm_height, &watermark.size)?;

    wand_wm.resize_image(
        wm_target_width as usize,
        wm_target_height as usize,
        FilterType_PointFilter,
    );
    let (left, top, right, bottom) = get_watermark_borders(
        wand.get_image_width() as i32,
        wand.get_image_height() as i32,
        wm_target_width,
        wm_target_height,
        &watermark.position,
        &watermark.origin,
    );
    debug!(
        "Watermark position - Padding: top: {}, left: {}, bottom: {}, right: {}",
        top, left, bottom, right
    );
    let mut pixel_wand = PixelWand::new();
    pixel_wand.set_color("transparent")?;
    unsafe {
        MagickTransparentPaintImage(wand_wm.wand, pixel_wand.wand, watermark.alpha, 0.0, 1);
    }

    wand.compose_images(
        &wand_wm,
        CompositeOperator_OverCompositeOp,
        true,
        left as isize,
        top as isize,
    )?;
    wand.write_image_blob(format!("{}", format).as_str())
        .map_err(|e| e.into())
}

fn rotate_image(img: &core::Mat, rotation: &Rotation) -> Result<core::Mat, opencv::Error> {
    let mut result_transpose = core::Mat::new()?;
    let mut result_flip = core::Mat::new()?;
    match rotation {
        Rotation::R90 => {
            core::transpose(&img, &mut result_transpose)?;
            core::flip(&result_transpose, &mut result_flip, 0)?;
        }
        Rotation::R180 => {
            core::flip(&img, &mut result_flip, -1)?;
        }
        Rotation::R270 => {
            core::transpose(&img, &mut result_transpose)?;
            core::flip(&result_transpose, &mut result_flip, 1)?;
        }
    }
    Ok(result_flip)
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
