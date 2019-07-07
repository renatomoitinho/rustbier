use actix_http::client::SendRequestError;
use actix_rt::System;
use actix_web::client::Client;
use bytes::Bytes;
use futures::future::lazy;
use futures::future::Future;
use std::fmt;
use std::fs::File;
use std::io::Read;

pub struct RequestParametersBuilder {
    filename: String,
    format: Option<ImageFormat>,
    quality: Option<i32>,
    w: Option<i32>,
    h: Option<i32>,
    wm_position: Option<WatermarkPosition>,
    wm_px: Option<i32>,
    wm_py: Option<i32>,
    wm_file: Option<String>,
    wm_alpha: Option<f64>,
    wm_h: Option<i32>,
    wm_w: Option<i32>,
}

pub enum WatermarkPosition {
    Center,
    LeftTop,
    RightBottom,
}

pub enum ImageFormat {
    Png,
    Jpeg,
    Webp,
}

impl RequestParametersBuilder {
    pub fn new(filename: String) -> Self {
        RequestParametersBuilder {
            filename,
            format: None,
            quality: None,
            w: None,
            h: None,
            wm_position: None,
            wm_px: None,
            wm_py: None,
            wm_file: None,
            wm_alpha: None,
            wm_h: None,
            wm_w: None,
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

    pub fn with_size(&mut self, width: i32, height: i32) -> &mut Self {
        self.w = Some(width);
        self.h = Some(height);
        self
    }

    pub fn with_watermark(
        &mut self,
        file: String,
        w: i32,
        h: i32,
        alpha: f64,
        x: i32,
        y: i32,
        pos: WatermarkPosition,
    ) -> &mut Self {
        self.wm_file = Some(file);
        self.wm_w = Some(w);
        self.wm_h = Some(h);
        self.wm_alpha = Some(alpha);
        self.wm_px = Some(x);
        self.wm_py = Some(y);
        self.wm_position = Some(pos);
        self
    }
}

pub fn get_results_file(filename: &str) -> Bytes {
    let mut file =
        File::open(format!("tests/resources/results/{}", filename)).expect("file does not exist");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("can't read file");
    buffer.into()
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
        query_string.push(format!("w={}", w));
    }
    if let Some(h) = params.h {
        query_string.push(format!("h={}", h));
    }
    if let Some(quality) = params.quality {
        query_string.push(format!("quality={}", quality));
    }
    if let Some(wm_file) = &params.wm_file {
        query_string.push(format!("wm_file={}", wm_file));
    }
    if let Some(wm_alpha) = params.wm_alpha {
        query_string.push(format!("wm_alpha={}", wm_alpha));
    }
    if let Some(wm_h) = params.wm_h {
        query_string.push(format!("wm_h={}", wm_h));
    }
    if let Some(wm_w) = params.wm_w {
        query_string.push(format!("wm_w={}", wm_w));
    }
    if let Some(wm_position) = &params.wm_position {
        query_string.push(format!("wm_position={}", wm_position));
    }
    if let Some(wm_px) = params.wm_px {
        query_string.push(format!("wm_px={}", wm_px));
    }
    if let Some(wm_py) = params.wm_py {
        query_string.push(format!("wm_py={}", wm_py));
    }
    format!(
        "http://localhost:8080/{}?{}",
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
