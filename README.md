# RustBier :beer:

This application runs a web server which performs image transformations.
The application supports:
* Retrieving source images from S3
* Encoding images to PNG, JPEG and WEBP
* Resizing an image
* Apply a watermark image to an image

## Configuration

A config file is required (`config/default.json`) for the application with the following format:

```json
{
    "png_quality": 3,
    "bucket": "apollo",
    "app_port": 8080,
    "region": "EuWest1",
    "log_level": "info"
}
```
| Name | Description | Required | Possible Values | Notes |
|------|-------------|----------|-----------------|-------|
| `log_level` | Logging level for the application | N | <ul><li>`error`</li><li>`warn`</li><li>`info`</li><li>`debug`</li><li>`trace`</li></ul> | Default value is `info`. |
| `bucket` | S3 source bucket for images  | Y | - | |
| `app_port` | Port which the web server listens to for requests  | Y | - | |
| `png_quality`| The PNG compression level for images encoded in this format. | Y | 0-9 | This setting impacts performance of the encoder and a higher value means a smaller size and longer compression time. |
| `region` | S3 region where the source bucket for images is located  | Y | <ul><li>`ApEast1`</li><li>`ApNortheast1`</li><li>`ApNortheast2`</li><li>`ApSouth1`</li><li>`ApSoutheast1`</li><li>`ApSoutheast2`</li><li>`CaCentral1`</li><li>`EuCentral1`</li><li>`EuWest1`</li><li>`EuWest2`</li><li>`EuWest3`</li><li>`EuNorth1`</li><li>`SaEast1`</li><li>`UsEast1`</li><li>`UsEast2`</li><li>`UsWest1`</li><li>`UsWest2`</li><li>`UsGovEast1`</li><li>`UsGovWest1`</li><li>`CnNorth1`</li><li>`CnNorthwest1`</li><li>`Custom`</li></ul> | When a `Custom` region is set, the configuration requires an endpoint and region name to be specified. Example shown in the following section. |


### Specifying a custom image source
To specify a custom image source specify `"region": "Custom"`. The configuration requires an endpoint and region name. For example:
```json
{
  "png_quality": 3,
  "bucket": "apollo",
  "app_port": 8080,
  "log_level": "debug",
  "region": {
    "Custom": {
      "name": "dev",
      "endpoint": "http://localhost:9000"
    }
  }
}
```

## Running locally

### Requirements
* OpenCV
* Minio - S3 compatible container
* Docker
* Cargo

This application relies on OpenCV C++ library. That means it has to be previously installed into the system before compiling and/or running.

For Linux installation, follow [this instructions](https://docs.opencv.org/master/d7/d9f/tutorial_linux_install.html).

For Mac installation, run `brew install opencv`

It is necessary to have the env variable `PKG_CONFIG_PATH` set to build the application in Mac. `PKG_CONFIG_PATH=/usr/local/Cellar/opencv@3/3.4.5_2/lib/pkgconfig/` - location might change based on OpenCV version.

Also a S3 compatible container will be required to run the application.

```docker run -it -p 9000:9000 minio/minio server /data```

To build and run the application, run the following command:

``` cargo run ```

It is also possible to run both components inside docker:

```docker-compose up```

AWS credentials should be configured [following the doc](https://github.com/rusoto/rusoto/blob/master/AWS-CREDENTIALS.md).

## Testing

At the moment, the only logic the application has is regarding resizing and watermark positioning. There are some unit tests around these use cases in the package `image_processor`. All other functions are deeply tied to the `opencv` library which complicates testing since it is not easily mockable.

## API

The application supports the following endpoints.

### `/health`
Signifies the application is healthy by returning a HTTP Status OK - 200 return code.

### `/metrics`
Prometheus formatted metrics. Currently exposes request count and duration per endpoint

### `/{file_name}`
Fetches and processes an image file.

The `/{file_name}` endpoint takes a filename as path parameter and has optional query parameters described in more detail below.

#### General query parameters
| Parameter | Description |
|-----------------|-------------|
| `format` | desired image format. Possible values are Jpeg, Png and Webp. Defaults to Jpeg |
| `quality` | desired quality for the image. For Jpeg, it goes from 0 to 100 (defaults to 100). For Webp, it goes from 1 to 100 (defaults to 100). For Png, it will be ignored. |
| `w` | desired width for the image. Images won't get upscaled or have their aspect ratio changed by variations on parameters for width and height. |
| `h` | desired height for the image. Images won't get upscaled or have their aspect ratio changed by variations on parameters for width and height. |
 
#### Watermarking query parameters
| Parameter | Description |
|-----------------|-------------|
| `wm_file` | watermark file. File has to be smaller than original file. |
| `wm_alpha` | opacity from the watermark over the original image. it is a floating point number from 0 to 1. |
| `wm_position` | identifier to position the watermark starting from left-top, right-bottom or if it should be centered (`wm_px` and `wm_py` will be ignored in that case). Possible values: LeftTop (default), RightBottom, Center. |
| `wm_px` | position of the watermark in the X axis. Value in pixels. |
| `wm_py` | position of the watermark in the Y axis. Value in pixels. |
| `wm_h` | optional height of the watermark. Same resizing rules from original image applies for watermark images. |
| `wm_w` | optional width of the watermark. Same resizing rules from original image applies for watermark images. |


## Conclusion

YES! We love beer! :beers: