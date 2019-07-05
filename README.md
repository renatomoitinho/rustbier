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

``` cargo run --bin rustbier ```

The parameter `--bin` has to be specified because there's a second binary in this application. The other implementation works with futures and async http requests (it will likely be deleted due to being less effective).

AWS credentials should be informed [following the doc](https://github.com/rusoto/rusoto/blob/master/AWS-CREDENTIALS.md).
