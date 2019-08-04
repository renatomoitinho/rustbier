use actix_web::web::Bytes;
use actix_web::Error;
use futures::future::Future;
use futures::Stream;
use rusoto_core::RusotoError;
use rusoto_s3::{GetObjectError, GetObjectRequest, S3Client, S3};

pub fn get_image(
    client: &S3Client,
    bucket: &str,
    filename: &str,
) -> impl Future<Item = Bytes, Error = Error> {
    info!("Fetching image {} from S3 bucket: {}", filename, bucket);
    client
        .get_object(GetObjectRequest {
            bucket: bucket.to_string(),
            key: filename.to_string(),
            ..Default::default()
        })
        .map_err(|e| match e {
            RusotoError::Service(GetObjectError::NoSuchKey(key)) => {
                actix_web::error::ErrorNotFound(format!("File {} not found", key))
            }
            e => {
                error!("Error fetching file from S3: {:?}", e);
                actix_web::error::ErrorInternalServerError(e)
            }
        })
        .map(|res| {
            info!("Response {:?}", res);
            let stream = res.body.expect("Error retrieving the body stream");
            stream.concat2().map_err(|e| {
                error!("Error fetching file from S3: {:?}", e);
                actix_web::error::ErrorInternalServerError(e)
            })
        })
        .flatten()
}
