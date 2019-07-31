pub mod errors;

use config::{Config, ConfigError, File};
use errors::InvalidSizeError;
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
    pub rotation: Option<Rotation>,
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

#[derive(Debug, Deserialize, Clone, Copy)]
pub enum Rotation {
    R90,
    R180,
    R270,
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

fn get_ratio(desired_measure: i32, original_measure: i32, opposite_orig_measure: i32) -> i32 {
    let ratio = desired_measure as f32 / original_measure as f32;
    (opposite_orig_measure as f32 * ratio) as i32
}

fn is_negative_or_zero(size: &Size) -> bool {
    (size.height.is_some() && size.height.unwrap() <= 0)
        || (size.width.is_some() && size.width.unwrap() <= 0)
}

pub fn get_target_size(
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

pub fn get_watermark_borders(
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
