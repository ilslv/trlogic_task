use actix_web::{HttpResponse, web, Result};
use actix_multipart::{Multipart, Field};
use futures::{StreamExt, TryStreamExt};
use uuid::Uuid;
use std::io::Write;
use serde::{Serialize, Deserialize};
use mime::Mime;

pub(crate) async fn multipart_handler(mut payload: Multipart) -> Result<HttpResponse> {
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = Field::content_type(&field);
        if content_type.type_() == mime::IMAGE {
            let filepath = format!("images/{}.{}", Uuid::new_v4(), content_type.subtype());

            let mut file = web::block(|| std::fs::File::create(filepath))
                .await?;

            while let Some(Ok(chunk)) = field.next().await {
                file = web::block(move || file.write_all(&chunk).map(|_| file)).await?;
            }
        }
    }

    Ok(HttpResponse::Ok().into())
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Image {
    Url(String),
    Base64(String),
}

fn get_mime(resp: &reqwest::Response) -> Option<Mime> {
    let content_type = resp
        .headers()
        .get("content-type")?
        .to_str().ok()?;

    content_type.parse::<Mime>().ok()
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
                        let filepath = format!("images/{}.{}", Uuid::new_v4(), resp_mime.subtype());

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
                //TODO: base64 handling
            }
        }
    }

    Ok(HttpResponse::Ok().into())
}