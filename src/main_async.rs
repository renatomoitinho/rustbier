#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate actix_web;
extern crate actix_web_prom;
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
use futures::future::Either;
use futures::prelude::Future;
use futures::Stream;
use image_processor::*;
use rusoto_core::{Region, RusotoError};
use rusoto_s3::{GetObjectError, GetObjectRequest, S3Client, S3};
use std::env;
use std::io::Read;

#[derive(Debug, Deserialize)]
struct QueryParameters {
    format: Option<ImageFormat>,
    quality: Option<i32>,
    w: Option<i32>,
    h: Option<i32>,
    wm_px: Option<i32>,
    wm_py: Option<i32>,
    wm_file: Option<String>,
    wm_alpha: Option<f64>,
    wm_h: Option<i32>,
    wm_w: Option<i32>,
}

fn index(
    _req: HttpRequest,
    path: web::Path<String>,
    query: web::Query<QueryParameters>,
    s3: web::Data<S3Client>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let wm_buffer = if let Some(file) = &query.wm_file {
        let s3_res = s3
            .get_object(GetObjectRequest {
                bucket: "apollo".to_owned(),
                key: file.to_owned(),
                ..Default::default()
            })
            .sync();
        if s3_res.is_err() {
            let e = s3_res.unwrap_err();
            error!("Error processing request: {:?}", e);
            return Either::A(futures::future::err(
                actix_web::error::ErrorInternalServerError(e),
            ));
        }
        let s3_stream = s3_res
            .unwrap()
            .body
            .expect("Error retrieving the body stream from watermark");
        let mut wm = Vec::new();
        s3_stream
            .into_blocking_read()
            .read_to_end(&mut wm)
            .expect("Error reading the body stream from watermark");
        Some(wm)
    } else {
        None
    };
    let fmt = query.format.unwrap_or(ImageFormat::Jpeg).clone();
    Either::B(
        s3.get_object(GetObjectRequest {
            bucket: "apollo".to_owned(),
            key: path.to_owned(),
            ..Default::default()
        })
        .and_then(move |res| {
            info!("Response {:?}", res);
            let stream = res.body.expect("Error retrieving the body stream");
            stream
                .concat2()
                .map(move |body| {
                    process_image(
                        &body[..],
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
                                    alpha: query.wm_alpha.unwrap_or_default(),
                                    size: Size {
                                        width: query.wm_w,
                                        height: query.wm_h,
                                    },
                                }),
                            },
                        },
                        3
                    )
                    .map_err(|e| actix_web::error::ErrorInternalServerError(e))
                })
                .map_err(|e| RusotoError::from(e))
        })
        .map_err(|e| match e {
            RusotoError::Service(GetObjectError::NoSuchKey(key)) => {
                actix_web::error::ErrorNotFound(format!("{} not found", key))
            }
            error => {
                error!("Error: {:?}", error);
                actix_web::error::ErrorInternalServerError("Error processing request")
            }
        })
        .from_err()
        .and_then(move |res| match res {
            Ok(img) => HttpResponse::Ok()
                .content_type(format!("image/{}", fmt).as_str())
                .body(Body::from(img)),
            Err(e) => {
                error!("Error processing request: {:?}", e);
                HttpResponse::from_error(e)
            }
        }),
    )
}

fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
}

fn main() -> std::io::Result<()> {
    let name = "rust_opencv";
    env::set_var("RUST_LOG", "trace");
    env::set_var(
        "AWS_SECRET_ACCESS_KEY",
        "aeacqJHFQV3xQgxGP95U4O7KyEKJgaejunGW7fum",
    );
    env::set_var("AWS_ACCESS_KEY_ID", "1PF4C4NPLNX1K38PWHQ8");
    pretty_env_logger::init();
    let sys = actix_rt::System::builder().stop_on_panic(false).build();
    let prometheus = PrometheusMetrics::new(name, "/metrics");
    let s3 = S3Client::new(Region::Custom {
        name: "eu-west-1".to_owned(),
        endpoint: "http://localhost:9000".to_owned(),
    });
    let s3_client_data = web::Data::new(s3);

    Server::build()
        .bind(name, "0.0.0.0:8080", move || {
            HttpService::build().keep_alive(KeepAlive::Os).h1(App::new()
                .register_data(s3_client_data.clone())
                .wrap(prometheus.clone())
                .wrap(actix_web::middleware::Logger::default())
                .service(web::resource("/health").route(web::get().to(health)))
                .service(web::resource("/{file_name}").route(web::get().to_async(index))))
        })?
        .start();
    sys.run()
}
