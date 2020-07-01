use actix_web::{HttpResponse, web, Result};
use actix_multipart::{Multipart, Field};
use futures::{StreamExt, TryStreamExt};
use uuid::Uuid;
use std::io::Write;
use serde::{Serialize, Deserialize};
use mime::Mime;
use regex::Regex;
use lazy_static::lazy_static;
use image::ImageError;
use crate::{FULL_IMG_PATH, PREVIEW_IMG_PATH};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Image {
    Url(String),
    Base64(String),
}

struct DataURL {
    type_: Mime,
    data: Vec<u8>,
}

fn parse_data_url(data_url: &str) -> Option<DataURL> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"data:(?P<type>.+?);base64,(?P<data>.+)").unwrap();
    }

    let captures = RE.captures(data_url)?;
    let type_mime = captures.name("type")?.as_str().parse::<Mime>().ok()?;
    let data = captures.name("data")?.as_str().as_bytes();

    Some(DataURL {
        type_: type_mime,
        data: Vec::from(data),
    })
}

fn get_mime(resp: &reqwest::Response) -> Option<Mime> {
    let content_type = resp
        .headers()
        .get("content-type")?
        .to_str().ok()?;

    content_type.parse::<Mime>().ok()
}

fn uuid_filename(file_extension: &Mime) -> String {
    format!("{}.{}", Uuid::new_v4(), file_extension.subtype().as_str())
}

fn resize_image(file_mime: &str) -> Result<(), ImageError> {
    let preview_img_path = format!("{}{}", PREVIEW_IMG_PATH, file_mime);
    image::open(format!("images/full/{}", file_mime))?
        .resize(100, 100, image::imageops::Nearest)
        .save(preview_img_path)?;
    Ok(())
}

pub(crate) async fn multipart_handler(mut payload: Multipart) -> Result<HttpResponse> {
    let mut response = Vec::new();

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = (*Field::content_type(&field)).clone();
        if content_type.type_() == mime::IMAGE {
            let filename = uuid_filename(&content_type);

            let (mut file, filename) = web::block(move || {
                std::fs::File::create(format!("{}{}", FULL_IMG_PATH, filename))
                    .map(|file| (file, filename))
            }).await?;

            while let Some(Ok(chunk)) = field.next().await {
                file = web::block(move || file.write_all(&chunk).map(|_| file)).await?;
            }

            let filename = web::block(move || -> Result<String, ImageError> {
                resize_image(&filename)?;
                Ok(filename)
            }).await?;

            println!("multipart: {}", &filename);
            response.push(filename);
        }
    }

    Ok(HttpResponse::Ok().json(response))
}

pub(crate) async fn json_handler(images: web::Json<Vec<Image>>) -> Result<HttpResponse> {
    let mut response = Vec::new();
    let client = reqwest::Client::new();

    for image in images.0 {
        match image {
            Image::Url(url) => {
                let resp = client
                    .get(&url)
                    .send()
                    .await
                    .map_err(|_| HttpResponse::BadRequest())?;

                if let Some(resp_mime) = get_mime(&resp) {
                    if resp_mime.type_() == mime::IMAGE {
                        let filename = uuid_filename(&resp_mime);

                        let (mut file, filename) = web::block(move || {
                            std::fs::File::create(format!("{}{}", FULL_IMG_PATH, &filename))
                                .map(|file| (file, filename))
                        }).await?;

                        let mut image_stream = resp.bytes_stream();
                        while let Some(Ok(chunk)) = image_stream.next().await {
                            file = web::block(move || file.write_all(&chunk).map(|_| file)).await?;
                        }

                        let filename = web::block(move || -> Result<String, ImageError> {
                            resize_image(&filename)?;
                            Ok(filename)
                        }).await?;

                        println!("url: {}", &filename);
                        response.push(filename);
                    }
                }
            },
            Image::Base64(base64_str) => {
                if let Some(data_url) = parse_data_url(&base64_str) {
                    if data_url.type_.type_() == mime::IMAGE {
                        let filename = uuid_filename(&data_url.type_);


                        let (mut file, filename) = web::block(move || {
                            std::fs::File::create(format!("{}{}", FULL_IMG_PATH, filename))
                                .map(|file| (file, filename))
                        }).await?;

                        let buf = web::block(move || base64::decode(&data_url.data))
                            .await?;

                        web::block(move || file.write_all(&buf))
                            .await?;

                        let filename = web::block(move || -> Result<String, ImageError> {
                            resize_image(&filename)?;
                            Ok(filename)
                        }).await?;

                        println!("base64: {}", &filename);
                        response.push(filename);
                    }
                }
            }
        }
    }

    Ok(HttpResponse::Ok().json(response))
}
