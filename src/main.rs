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
extern crate serde_qs;

mod commons;
mod image_processor;

use commons::s3;
use commons::*;

use actix_http::{HttpService, KeepAlive};
use actix_server::Server;
use actix_web::{dev::Body, web, App, HttpRequest, HttpResponse};
use actix_web_prom::PrometheusMetrics;
use futures::future::{join_all, Future};
use image_processor::*;
use magick_rust::magick_wand_genesis;
use rusoto_s3::S3Client;
use std::env;
use std::sync::{Arc, Once};

static START: Once = Once::new();

fn index(
    req: HttpRequest,
    path: web::Path<String>,
    qs_config: web::Data<serde_qs::Config>,
    s3_client: web::Data<S3Client>,
    config: web::Data<Configuration>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let rs_query = qs_config
        .deserialize_str::<ProcessImageRequest>(req.query_string())
        .map_err(actix_web::error::ErrorBadRequest);
    futures::done(rs_query).and_then(move |query| {
        debug!("Request parameters: {:?}", query);

        let ProcessImageRequest {
            size,
            format,
            quality,
            watermarks,
            rotation,
        } = query;
        let bucket = Arc::new(config.bucket.clone());
        let bucket_cp = bucket.clone();
        let s3_client_cp = s3_client.clone();
        let wm_futures = watermarks
            .clone()
            .into_iter()
            .map(move |wm| s3::get_image(&s3_client_cp, &bucket_cp, &wm.filename));
        s3::get_image(&s3_client, &bucket, &path)
            .map(move |body| {
                pre_process_image(
                    &body[..],
                    rotation,
                    &size,
                    format,
                    quality,
                    config.png_quality,
                )
                .map_err(|e| {
                    error!("Error processing image: {:?}", e);
                    actix_web::error::ErrorInternalServerError(e)
                })
            })
            .map(move |body| {
                join_all(wm_futures).map(move |buffers| {
                    buffers
                        .iter()
                        .zip(watermarks)
                        .fold(body, move |current, item| {
                            let (wm_buffer, wm) = item;
                            apply_watermark(&current?, &wm_buffer[..], &wm, format)
                                .map_err(|e| e.into())
                        })
                })
            })
            .flatten()
            .and_then(move |res| index_response(res, format))
    })
}

fn index_response(
    res: Result<Vec<u8>, actix_web::error::Error>,
    format: ImageFormat,
) -> HttpResponse {
    match res {
        Err(e) => {
            error!("Error processing request: {:?}", e);
            HttpResponse::from_error(e)
        }
        Ok(img_response) => HttpResponse::Ok()
            .content_type(format!("image/{}", format).as_str())
            .body(Body::from(img_response)),
    }
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
    //accept url encoded with brackets or their encoded equivalents
    let qs_config = serde_qs::Config::new(5, false);
    let qs_config_data = web::Data::new(qs_config);

    Server::build()
        .bind(
            name,
            format!("0.0.0.0:{}", config_data.app_port),
            move || {
                HttpService::build().keep_alive(KeepAlive::Os).h1(App::new()
                    .register_data(s3_client_data.clone())
                    .register_data(config_data.clone())
                    .register_data(qs_config_data.clone())
                    .wrap(prometheus.clone())
                    .wrap(actix_web::middleware::Logger::default())
                    .service(web::resource("/health").route(web::get().to(health)))
                    .service(web::resource("/test").route(web::get().to(test)))
                    .service(web::resource("/{file_name}").route(web::get().to_async(index))))
            },
        )?
        .start();
    sys.run()
}
