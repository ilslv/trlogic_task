use actix_web::{HttpResponse, web, Result};
use actix_multipart::{Multipart, Field};
use futures::{StreamExt, TryStreamExt};
use uuid::Uuid;
use std::io::Write;
use serde::{Serialize, Deserialize};
use mime::Mime;
use regex::Regex;
use lazy_static::lazy_static;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Image {
    Url(String),
    Base64(String),
}

struct DataURL {
    type_: Mime,
    data: Vec<u8>
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

fn uuid_filepath(file_extension: &str) -> String {
    format!("images/{}.{}", Uuid::new_v4(), file_extension)
}

pub(crate) async fn multipart_handler(mut payload: Multipart) -> Result<HttpResponse> {
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = Field::content_type(&field);
        if content_type.type_() == mime::IMAGE {
            let filepath = uuid_filepath(content_type.subtype().as_str());

            println!("multipart: {}", filepath);

            let mut file = web::block(|| std::fs::File::create(filepath))
                .await?;

            while let Some(Ok(chunk)) = field.next().await {
                file = web::block(move || file.write_all(&chunk).map(|_| file)).await?;
            }

        }
    }

    Ok(HttpResponse::Ok().into())
}

pub(crate) async fn json_handler(images: web::Json<Vec<Image>>) -> Result<HttpResponse> {
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
                        let filepath = uuid_filepath(resp_mime.subtype().as_str());

                        println!("url: {}", filepath);

                        let mut file = web::block(|| std::fs::File::create(filepath))
                            .await?;

                        let mut image_stream = resp.bytes_stream();
                        while let Some(Ok(chunk)) = image_stream.next().await {
                            file = web::block(move || file.write_all(&chunk).map(|_| file)).await?;
                        }

                    }
                }
            }
            Image::Base64(base64_str) => {
                if let Some(data_url) = parse_data_url(&base64_str) {
                    if data_url.type_.type_() == mime::IMAGE {
                        let filepath = uuid_filepath(data_url.type_.subtype().as_str());

                        println!("base64: {}", filepath);

                        let mut file = web::block(|| std::fs::File::create(filepath))
                            .await?;

                        let buf = web::block(move || base64::decode(&data_url.data))
                            .await?;

                        web::block(move || file.write_all(&buf))
                            .await?;
                    }
                }
            }
        }
    }

    Ok(HttpResponse::Ok().into())
}
