#![feature(test)]
extern crate test;
use actix_rt::System;
use actix_web::client::Client;
use futures::future::lazy;
use futures::future::Future;
use test::Bencher;

#[bench]
fn bench(bencher: &mut Bencher) {
    bencher.iter(|| {
        System::new("test").block_on(lazy(|| {
            let client = Client::default();
            client
                .get("http://127.0.0.1:8080/highres?w=500&wm_file=watermark&wm_px=100&wm_py=200&wm_alpha=0.4")
                .header("User-Agent", "Actix-web")
                .send()
                .map_err(|e| panic!("request error: {}", e))
                .map(|mut response| {
                    println!("Response: {:?}", response);
                    response.body().limit(1024 * 1024).map_err(|e| panic!("error: {}", e))
                })
                .flatten()
        })).expect("Unable to download file")
    });
}
