#![feature(test)]
extern crate test;
use actix_rt::System;
use actix_web::client::Client;
use futures::future::lazy;
use futures::future::Future;
use test::Bencher;

#[bench]
fn bench_simple(bencher: &mut Bencher) {
    bencher.iter(|| {
        System::new("test")
            .block_on(lazy(|| {
                let client = Client::default();
                client
                    .get("http://127.0.0.1:8080/highres?size[width]=500&quality=90&rotation=R90")
                    .header("User-Agent", "Actix-web")
                    .send()
                    .map_err(|e| panic!("request error: {}", e))
                    .map(|mut response| {
                        println!("Response: {:?}", response);
                        response
                            .body()
                            .limit(1024 * 1024)
                            .map_err(|e| panic!("error: {}", e))
                    })
                    .flatten()
            }))
            .expect("Unable to download file")
    });
}

#[bench]
fn bench_single_watermark(bencher: &mut Bencher) {
    bencher.iter(|| {
        System::new("test").block_on(lazy(|| {
            let client = Client::default();
            client
                .get("http://127.0.0.1:8080/highres?size[width]=500&quality=90&watermarks[0][filename]=watermark&watermarks[0][alpha]=0.5")
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

#[bench]
fn bench_multiple_watermark(bencher: &mut Bencher) {
    bencher.iter(|| {
        System::new("test").block_on(lazy(|| {
            let client = Client::default();
            client
                .get("http://127.0.0.1:8080/highres?size[width]=500&quality=90&watermarks[0][filename]=watermark&watermarks[0][alpha]=0.5&watermarks[1][filename]=watermark&watermarks[1][alpha]=0.5&watermarks[1][origin]=Center&watermarks[2][filename]=watermark&watermarks[2][alpha]=0.5&watermarks[2][origin]=RightBottom")
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
