#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate actix_web;
extern crate actix_web_prom;
extern crate config;
extern crate futures;
extern crate opencv;
extern crate pretty_env_logger;
extern crate rusoto_s3;

mod commons;
mod image_processor;

use actix_http::{HttpService, KeepAlive};
use actix_server::Server;
use actix_web::{dev::Body, web, App, HttpRequest, HttpResponse};
use actix_web_prom::PrometheusMetrics;
use commons::*;
use image_processor::*;
use magick_rust::magick_wand_genesis;
use rusoto_core::RusotoError;
use rusoto_s3::{GetObjectError, GetObjectRequest, S3Client, S3};
use std::env;
use std::io::Read;
use std::sync::Once;

static START: Once = Once::new();

#[derive(Debug, Deserialize)]
struct QueryParameters {
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
    r: Option<Rotation>,
}

fn index(
    _req: HttpRequest,
    path: web::Path<String>,
    query: web::Query<QueryParameters>,
    s3: web::Data<S3Client>,
    config: web::Data<Configuration>,
) -> actix_web::Result<HttpResponse> {
    info!(
        "Fetching image {} and watermark {} from S3",
        path,
        query.wm_file.as_ref().unwrap_or(&"-".to_string())
    );
    debug!("Request parameters: {:?}", query);
    let wm_buffer = if let Some(file) = &query.wm_file {
        let wm_obj = s3
            .get_object(GetObjectRequest {
                bucket: config.bucket.clone(),
                key: file.to_owned(),
                ..Default::default()
            })
            .sync()
            .map_err(|e| match e {
                RusotoError::Service(GetObjectError::NoSuchKey(key)) => {
                    actix_web::error::ErrorNotFound(format!("Watermark {} not found", key))
                }
                e => {
                    error!("Error fetching watermark from S3: {:?}", e);
                    actix_web::error::ErrorInternalServerError(e)
                }
            })?;
        let mut buf = Vec::new();
        let wm_stream = wm_obj
            .body
            .ok_or(actix_web::error::ErrorInternalServerError(
                "There was no stream for watermark",
            ))?;
        wm_stream
            .into_blocking_read()
            .read_to_end(&mut buf)
            .map_err(|e| {
                error!("Error reading watermark stream: {:?}", e);
                actix_web::error::ErrorInternalServerError(e)
            })?;
        Some(buf)
    } else {
        None
    };

    let file_obj = s3
        .get_object(GetObjectRequest {
            bucket: config.bucket.clone(),
            key: path.to_owned(),
            ..Default::default()
        })
        .sync()
        .map_err(|e| match e {
            RusotoError::Service(GetObjectError::NoSuchKey(key)) => {
                actix_web::error::ErrorNotFound(format!("{} not found", key))
            }
            e => {
                error!("Error fetching image from S3: {:?}", e);
                actix_web::error::ErrorInternalServerError(e)
            }
        })?;

    let mut file_buffer = Vec::new();
    let file_stream = file_obj
        .body
        .ok_or(actix_web::error::ErrorInternalServerError(
            "There was no stream for image",
        ))?;
    file_stream
        .into_blocking_read()
        .read_to_end(&mut file_buffer)
        .map_err(|e| {
            error!("Error reading stream: {:?}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;

    let fmt = query.format.unwrap_or(ImageFormat::Jpeg).clone();
    let img_response = process_image(
        file_buffer.as_slice(),
        ProcessImageRequest {
            size: Size {
                width: query.w,
                height: query.h,
            },
            quality: query.quality.unwrap_or(100),
            format: query.format.clone().unwrap_or(ImageFormat::Jpeg),
            watermark: match &wm_buffer {
                None => None,
                Some(f) => Some(Watermark {
                    file: f.as_slice(),
                    position: Point {
                        x: query.wm_px.unwrap_or_default(),
                        y: query.wm_py.unwrap_or_default(),
                    },
                    origin: query
                        .wm_position
                        .clone()
                        .unwrap_or(WatermarkPosition::LeftTop),
                    alpha: query.wm_alpha.unwrap_or_default(),
                    size: Size {
                        width: query.wm_w,
                        height: query.wm_h,
                    },
                }),
            },
            rotation: query.r,
        },
        config.png_quality,
    )
    .map_err(|e| {
        error!("Error processing image: {:?}", e);
        actix_web::error::ErrorInternalServerError(e)
    })?;

    Ok(HttpResponse::Ok()
        .content_type(format!("image/{}", fmt).as_str())
        .body(Body::from(img_response)))
}

fn test(req: HttpRequest) -> HttpResponse {
    println!("{:?}", req.query_string());
    HttpResponse::Ok().finish()
}

fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
}

fn main() -> std::io::Result<()> {
    START.call_once(|| {
        magick_wand_genesis();
    });
    let config = Configuration::new().expect("Failed to load application configuration.");
    let config_data = web::Data::new(config);
    let name = "rustbier";
    env::set_var(
        "RUST_LOG",
        config_data
            .log_level
            .as_ref()
            .unwrap_or(&"info".to_string()),
    );
    pretty_env_logger::init();
    let sys = actix_rt::System::builder().stop_on_panic(false).build();
    let prometheus = PrometheusMetrics::new(name, "/metrics");
    let s3 = S3Client::new(config_data.region.clone());
    let s3_client_data = web::Data::new(s3);

    Server::build()
        .bind(
            name,
            format!("0.0.0.0:{}", config_data.app_port),
            move || {
                HttpService::build().keep_alive(KeepAlive::Os).h1(App::new()
                    .register_data(s3_client_data.clone())
                    .register_data(config_data.clone())
                    .wrap(prometheus.clone())
                    .wrap(actix_web::middleware::Logger::default())
                    .service(web::resource("/health").route(web::get().to(health)))
                    .service(web::resource("/test").route(web::get().to(test)))
                    .service(web::resource("/{file_name}").route(web::get().to(index))))
            },
        )?
        .start();
    sys.run()
}
