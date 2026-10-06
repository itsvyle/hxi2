use axum::response::IntoResponse;
use axum::{Router, body::Bytes, routing::get};
use http::{HeaderValue, header};
use rust_embed::Embed;
use tracing::trace;

use crate::permissions_checking;
#[derive(Embed)]
#[allow_missing = true]
#[folder = "dist/"]
#[prefix = "dist/"]
struct EmbedAsset;

pub fn router() -> anyhow::Result<Router> {
    let mut router = Router::new();

    let v = EmbedAsset::iter().collect::<Vec<_>>();
    println!("Embedded files: {:?}", v);

    for (_, method_perms) in permissions_checking::get_compiled_permissions().permissions {
        if method_perms.is_frontend
            && let Some(file_path) = method_perms.frontend_static_file.to_owned()
        {
            trace!(
                "Registering static file route: {} -> {}",
                method_perms
                    .public_url
                    .map(|urls| urls.join(", "))
                    .unwrap_or_default(),
                file_path
            );
            let embedded_file = EmbedAsset::get(&file_path)
                .ok_or_else(|| anyhow::anyhow!("Failed to find embedded file: {}", file_path))?;

            // 1. Zero-copy static bytes (no .to_vec() heap copy)
            let body = match embedded_file.data {
                std::borrow::Cow::Borrowed(slice) => Bytes::from_static(slice),
                std::borrow::Cow::Owned(vec) => Bytes::from(vec),
            };

            // 2. Pre-parse MIME header as HeaderValue once
            let mime = mime_guess::from_path(&file_path).first_or_octet_stream();
            let content_type = HeaderValue::from_str(mime.as_ref()).unwrap();

            for url in method_perms.public_url.unwrap_or(&[]) {
                let body = body.clone(); // Cheap ref-count increment (or slice pointer)
                let content_type = content_type.clone();

                router = router.route(
                    url,
                    get(move || async move {
                        ([(header::CONTENT_TYPE, content_type)], body).into_response()
                    }),
                );
            }

            // for url in method_perms.public_url.unwrap_or(&[]) {
            //     app = app.route(
            //         url,
            //         get(move || async move {
            //             let file_content = tokio::fs::read(format!("./src/{file_path}"))
            //                 .await
            //                 .map_err(|err| {
            //                     error!(
            //                         error = %err,
            //                         file_path = "./src/{file_path}",
            //                         "Failed to read file from disk"
            //                     );
            //                     (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file")
            //                 })?;

            //             Ok::<_, (StatusCode, &'static str)>(Html(file_content))
            //         }),
            //     );
            //     debug!("Registered static file route: {} -> {}", url, file_path);
            // }
        }
    }

    Ok(router)
}
