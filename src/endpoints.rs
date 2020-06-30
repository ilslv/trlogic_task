use actix_web::{HttpResponse, web, Result};
use actix_multipart::{Multipart, Field};
use futures::{StreamExt, TryStreamExt};
use uuid::Uuid;
use std::io::Write;

pub(crate) async fn multipart_handler(mut payload: Multipart) -> Result<HttpResponse> {
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = Field::content_type(&field);
        if content_type.type_() == mime::IMAGE {
            let filepath = format!("images/{}.{}", Uuid::new_v4(), content_type.subtype());

            let mut file = web::block(|| std::fs::File::create(filepath))
                .await?;

            while let Some(chunk) = field.next().await {
                let data = chunk?;
                file = web::block(move || file.write_all(&data).map(|_| file)).await?;
            }
        }
    }

    Ok(HttpResponse::Ok().into())
}