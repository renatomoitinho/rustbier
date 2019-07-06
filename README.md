# RustBier

This application runs a web server serves images stored in S3, reencode to PNG, JPEG or WEBP, resize and add watermark to them.

## Configuration

It is required to create a config file (`config\default.json`) for the application having the following format:

```json
{
    "png_quality": 3,
    "bucket": "apollo",
    "app_port": 8080,
    "region": "EuWest1",
    "log_level": "info"
}
```

The only optional config is the log_level, which will have a default value of `info`. Other possible values are: `error`, `warn`, `debug` and `trace`.

The possible values for region are the following:
* ApEast1
* ApNortheast1
* ApNortheast2
* ApSouth1
* ApSoutheast1
* ApSoutheast2
* CaCentral1
* EuCentral1
* EuWest1
* EuWest2
* EuWest3
* EuNorth1
* SaEast1
* UsEast1
* UsEast2
* UsWest1
* UsWest2
* UsGovEast1
* UsGovWest1
* CnNorth1
* CnNorthwest1
* Custom

When `Custom` is set, the configuration changes slightly, because the endpoint and region name have to be specified. Here's an example: 

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

This application relies on OpenCV C++ library. That means it has to be previously installed into the system before compiling and/or running.
For Linux, follow [this instructions](https://docs.opencv.org/master/d7/d9f/tutorial_linux_install.html).

For Mac, run `brew install opencv`

It is necessary to have the env variable `PKG_CONFIG_PATH` set to build the application in Mac. `PKG_CONFIG_PATH=/usr/local/Cellar/opencv@3/3.4.5_2/lib/pkgconfig/` - location might change based on OpenCV version.

Also a S3 compatible container will be required to run the application.

```docker run -it -p 9000:9000 minio/minio server /data```

To build and run the application, run the following command:

``` cargo run ```

It is also possible to run both components inside docker:

```docker-compose up```

AWS credentials should be informed [following the doc](https://github.com/rusoto/rusoto/blob/master/AWS-CREDENTIALS.md).

## Testing

In the moment, the only logic the application has is regarding resizing and watermark positioning. There are some unit tests around that in the package `image_processor`. All ther other functions are deeply tied to `opencv` library, which makes harder to unit test, since it is not easily mockable.

## Endpoints

The application has 3 endpoints:

1. `/health` -> simply return `200` status to assure the application is healthy.
2. `/metrics` -> prometheus formated metrics. In the moment, it exposes only request count and duration per endpoint
3. `/{file_name}` -> fetches and processes a file.

The `/{file_name}` endpoint takes a filename as path paramenter and has the following optional query parameters:

* `format` -> desired image format. Possible values are Jpeg, Png and Webp. Defaults to Jpeg
* `quality` -> desired quality for the image. For Jpeg, it goes from 0 to 100 (defaults to 100). For Webp, it goes from 1 to 100 (defaults to 100). For Png, it will be ignored. Png takes only compression level as parameter, and, since it impacts performance, it is set at configuration level through the field `png_quality`. It goes from 0 to 9 and a higher value means a smaller size and longer compression time. 
* `w` -> desired width for the image. Images won't get upscaled or have their aspect ratio changed by variations on parameters for width and height.
* `h` -> desired height for the image. Images won't get upscaled or have their aspect ratio changed by variations on parameters for width and height.
* `wm_file` -> watermark file. File has to be smaller than original file.
* `wm_alpha` -> opacity from the watermark over the original image. it is a floating point number from 0 to 1.
* `wm_position` -> identifier to position the watermark starting from left-top, right-bottom or if it should be centered (`wm_px` and `wm_py` will be ignored in that case). Possible values: LeftTop (default), RightBottom, Center.
* `wm_px` -> position of the watermark in the X axis. Value in pixels.
* `wm_py` -> position of the watermark in the Y axis. Value in pixels.
* `wm_h` -> optional height of the watermark. Same resizing rules from original image applies for watermark images.
* `wm_w` -> optional width of the watermark. Same resizing rules from original image applies for watermark images.