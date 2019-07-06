pub mod errors;

use config::{Config, ConfigError, File};
use rusoto_core::Region;
use std::env;
use std::fmt;

#[derive(Serialize, Deserialize)]
#[serde(remote = "Region")]
pub enum RegionDef {
    ApEast1,
    ApNortheast1,
    ApNortheast2,
    ApSouth1,
    ApSoutheast1,
    ApSoutheast2,
    CaCentral1,
    EuCentral1,
    EuWest1,
    EuWest2,
    EuWest3,
    EuNorth1,
    SaEast1,
    UsEast1,
    UsEast2,
    UsWest1,
    UsWest2,
    UsGovEast1,
    UsGovWest1,
    CnNorth1,
    CnNorthwest1,
    Custom { name: String, endpoint: String },
}

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub png_quality: u8,
    #[serde(with = "RegionDef")]
    pub region: Region,
    pub bucket: String,
    pub app_port: u16,
    pub log_level: Option<String>,
}

#[derive(Debug)]
pub struct ProcessImageRequest<'a> {
    pub size: Size,
    pub format: ImageFormat,
    pub quality: i32,
    pub watermark: Option<Watermark<'a>>,
}

#[derive(Debug)]
pub struct Watermark<'a> {
    pub file: &'a [u8],
    pub position: Point,
    pub origin: WatermarkPosition,
    pub alpha: f64,
    pub size: Size,
}

#[derive(Debug)]
pub struct Size {
    pub width: Option<i32>,
    pub height: Option<i32>,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub enum WatermarkPosition {
    Center,
    LeftTop,
    RightBottom,
}

#[derive(Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Webp,
}

impl fmt::Display for ImageFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let as_str = match self {
            ImageFormat::Jpeg => "jpeg",
            ImageFormat::Png => "png",
            ImageFormat::Webp => "webp",
        };
        write!(f, "{}", as_str)
    }
}

impl Configuration {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();

        // Start off by merging in the "default" configuration file
        s.merge(File::with_name("config/default"))?;

        // Add in the current environment file
        // Default to 'development' env
        // Note that this file is _optional_
        let env = env::var("RUN_MODE").unwrap_or("development".into());
        s.merge(File::with_name(&format!("config/{}", env)).required(false))?;

        // Deserialize (and thus freeze) the entire configuration as
        s.try_into()
    }
}
