mod utils;

#[test]
fn test_get_simple() {
    let expected = utils::get_results_file("raw.jpg");
    let result = utils::make_request(&utils::RequestParametersBuilder::new(
        "img-test".to_string(),
    ))
    .expect("Unable to download file");
    assert!(expected == result);
}

#[test]
fn test_get_resized() {
    let expected = utils::get_results_file("resized.jpg");
    let result = utils::make_request(
        &utils::RequestParametersBuilder::new("img-test".to_string()).with_size(100, 100),
    )
    .expect("Unable to download file");
    assert!(expected == result);
}

#[test]
fn test_get_watermarked_left() {
    let expected = utils::get_results_file("watermarked_left.jpg");
    let result = utils::make_request(
        &utils::RequestParametersBuilder::new("img-test".to_string()).with_watermark(
            "watermark".to_string(),
            100,
            100,
            0.5f64,
            10,
            10,
            utils::WatermarkPosition::LeftTop,
        ),
    )
    .expect("Unable to download file");
    assert!(expected == result);
}

#[test]
fn test_get_watermarked_right() {
    let expected = utils::get_results_file("watermarked_right.jpg");
    let result = utils::make_request(
        &utils::RequestParametersBuilder::new("img-test".to_string()).with_watermark(
            "watermark".to_string(),
            100,
            100,
            0.5f64,
            10,
            10,
            utils::WatermarkPosition::RightBottom,
        ),
    )
    .expect("Unable to download file");
    assert!(expected == result);
}

#[test]
fn test_get_watermarked_center() {
    let expected = utils::get_results_file("watermarked_center.jpg");
    let result = utils::make_request(
        &utils::RequestParametersBuilder::new("img-test".to_string()).with_watermark(
            "watermark".to_string(),
            100,
            100,
            0.5f64,
            10,
            10,
            utils::WatermarkPosition::Center,
        ),
    )
    .expect("Unable to download file");
    assert!(expected == result);
}

#[test]
fn test_get_encoded_png() {
    let expected = utils::get_results_file("raw.png");
    let result = utils::make_request(
        &utils::RequestParametersBuilder::new("img-test".to_string())
            .with_format(utils::ImageFormat::Png),
    )
    .expect("Unable to download file");
    assert!(expected == result);
}

#[test]
fn test_get_encoded_webp() {
    let expected = utils::get_results_file("raw.webp");
    let result = utils::make_request(
        &utils::RequestParametersBuilder::new("img-test".to_string())
            .with_format(utils::ImageFormat::Webp),
    )
    .expect("Unable to download file");
    assert!(expected == result);
}

#[test]
fn test_get_encoded_webp_bad_quality() {
    let expected = utils::get_results_file("raw_bad_quality.webp");
    let result = utils::make_request(
        &utils::RequestParametersBuilder::new("img-test".to_string())
            .with_format(utils::ImageFormat::Webp)
            .with_quality(10),
    )
    .expect("Unable to download file");
    assert!(expected == result);
}

#[test]
fn test_get_raw_bad_quality() {
    let expected = utils::get_results_file("raw_bad_quality.jpg");
    let result = utils::make_request(
        &utils::RequestParametersBuilder::new("img-test".to_string())
            .with_format(utils::ImageFormat::Jpeg)
            .with_quality(10),
    )
    .expect("Unable to download file");
    assert!(expected == result);
}

#[test]
fn test_get_all_features() {
    let expected = utils::get_results_file("all_features.webp");
    let result = utils::make_request(
        &utils::RequestParametersBuilder::new("img-test".to_string())
            .with_format(utils::ImageFormat::Jpeg)
            .with_quality(50)
            .with_watermark(
                "watermark".to_string(),
                50,
                50,
                0.3f64,
                10,
                10,
                utils::WatermarkPosition::RightBottom,
            )
            .with_size(150, 150),
    )
    .expect("Unable to download file");
    assert!(expected == result);
}
