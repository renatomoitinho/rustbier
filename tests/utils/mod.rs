use actix_http::client::SendRequestError;
use actix_rt::System;
use actix_web::client::Client;
use bytes::Bytes;
use futures::future::lazy;
use futures::future::Future;
use magick_rust::bindings::MetricType_PerceptualHashErrorMetric;
use magick_rust::{magick_wand_genesis, MagickWand};
use std::env;
use std::fmt;
use std::sync::Once;

static START: Once = Once::new();

pub struct RequestParametersBuilder {
    filename: String,
    format: Option<ImageFormat>,
    quality: Option<i32>,
    w: Option<i32>,
    h: Option<i32>,
    watermarks: Vec<Watermark>,
    r: Option<Rotation>,
}

pub struct Watermark {
    filename: String,
    x: i32,
    y: i32,
    origin: WatermarkPosition,
    alpha: f64,
    w: i32,
    h: i32,
}

pub enum WatermarkPosition {
    Center,
    LeftTop,
    RightBottom,
}

pub enum Rotation {
    R90,
    R180,
    R270,
}

pub enum ImageFormat {
    Png,
    Jpeg,
    Webp,
}

impl RequestParametersBuilder {
    pub fn new(filename: &str) -> Self {
        RequestParametersBuilder {
            filename: filename.to_string(),
            format: None,
            quality: None,
            w: None,
            h: None,
            watermarks: Vec::new(),
            r: None,
        }
    }

    pub fn with_format(&mut self, format: ImageFormat) -> &mut Self {
        self.format = Some(format);
        self
    }

    pub fn with_quality(&mut self, quality: i32) -> &mut Self {
        self.quality = Some(quality);
        self
    }

    pub fn with_rotation(&mut self, rotation: Rotation) -> &mut Self {
        self.r = Some(rotation);
        self
    }

    pub fn with_size(&mut self, width: i32, height: i32) -> &mut Self {
        self.w = Some(width);
        self.h = Some(height);
        self
    }

    pub fn add_watermark(
        &mut self,
        file: &str,
        w: i32,
        h: i32,
        alpha: f64,
        x: i32,
        y: i32,
        pos: WatermarkPosition,
    ) -> &mut Self {
        self.watermarks.push(Watermark {
            filename: file.to_string(),
            origin: pos,
            x,
            y,
            alpha,
            w,
            h,
        });
        self
    }
}

pub fn assert_result(img: &[u8], filename: &str) {
    START.call_once(|| {
        magick_wand_genesis();
    });
    let wand1 = MagickWand::new();
    wand1.read_image_blob(img).expect("Unable to read response image");
    let wand2 = MagickWand::new();
    let file_result = format!("tests/results/{}", filename);
    wand2.read_image(&file_result).expect("Unable to result image");

    let (diff, _res_wand) = wand1.compare_images(&wand2, MetricType_PerceptualHashErrorMetric);
    println!("Image diff: {}", diff);
    assert!(diff == 0.0);
}

pub fn make_request(params: &RequestParametersBuilder) -> Result<Bytes, SendRequestError> {
    System::new("test").block_on(lazy(|| {
        let client = Client::default();

        let url = get_url(&params);
        println!("URL: {}", url);

        client
            .get(url)
            .header("User-Agent", "Actix-web")
            .send()
            .map(|mut response| {
                println!("Response: {:?}", response);
                response.body().map_err(|e| panic!("error: {}", e))
            })
            .flatten()
    }))
}

fn get_url(params: &RequestParametersBuilder) -> String {
    let mut query_string = Vec::new();
    if let Some(format) = &params.format {
        query_string.push(format!("format={}", format));
    }
    if let Some(w) = params.w {
        query_string.push(format!("size[width]={}", w));
    }
    if let Some(h) = params.h {
        query_string.push(format!("size[height]={}", h));
    }
    if let Some(quality) = params.quality {
        query_string.push(format!("quality={}", quality));
    }
    if let Some(rotation) = &params.r {
        query_string.push(format!("rotation={}", rotation));
    }
    for (i, item) in params.watermarks.iter().enumerate() {
        query_string.push(format!("watermarks[{}][filename]={}", i, item.filename));
        query_string.push(format!("watermarks[{}][alpha]={}", i, item.alpha));
        query_string.push(format!("watermarks[{}][size][height]={}", i, item.h));
        query_string.push(format!("watermarks[{}][size][width]={}", i, item.w));
        query_string.push(format!("watermarks[{}][origin]={}", i, item.origin));
        query_string.push(format!("watermarks[{}][position][x]={}", i, item.x));
        query_string.push(format!("watermarks[{}][position][y]={}", i, item.y));
    }

    format!(
        "http://{}:8080/{}?{}",
        env::var("RUSTBIER_HOST").unwrap_or("localhost".into()),
        params.filename,
        query_string.join("&")
    )
}

impl fmt::Display for WatermarkPosition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let as_str = match self {
            WatermarkPosition::Center => "Center",
            WatermarkPosition::LeftTop => "LeftTop",
            WatermarkPosition::RightBottom => "RightBottom",
        };
        write!(f, "{}", as_str)
    }
}

impl fmt::Display for ImageFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let as_str = match self {
            ImageFormat::Jpeg => "Jpeg",
            ImageFormat::Png => "Png",
            ImageFormat::Webp => "Webp",
        };
        write!(f, "{}", as_str)
    }
}

impl fmt::Display for Rotation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let as_str = match self {
            Rotation::R90 => "R90",
            Rotation::R180 => "R180",
            Rotation::R270 => "R270",
        };
        write!(f, "{}", as_str)
    }
}
