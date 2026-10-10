use axum::response::IntoResponse;
use axum::{Router, body::Bytes, routing::get};
use http::{HeaderValue, header};
use rust_embed::Embed;
use tracing::{error, trace};

use crate::permissions_checking;
#[derive(Embed)]
#[allow_missing = true]
#[folder = "dist/"]
#[prefix = "dist/"]
struct EmbedAsset;

pub fn router() -> anyhow::Result<Router> {
    let mut router = Router::new();

    let _v = EmbedAsset::iter().collect::<Vec<_>>();
    for (_, method_perms) in permissions_checking::get_compiled_permissions().permissions {
        if method_perms.is_frontend
            && let Some(file_path) = method_perms.frontend_static_file.to_owned()
        {
            let urls = method_perms.public_url.unwrap_or_default();
            if urls.is_empty() {
                error!("No public_url specified for static file: {}", file_path);
                continue;
            }

            let link_files = &[
                (file_path.to_string(), ""),
                (format!("{}.br", file_path), ".br"),
                (format!("{}.gz", file_path), ".gz"),
            ]
            .into_iter()
            .filter(|f| EmbedAsset::get(f.0.as_str()).is_some())
            .collect::<Vec<_>>();

            trace!(
                "Registering static file route: {} -> {}",
                urls.join(", "),
                link_files
                    .iter()
                    .map(|f| f.0.clone())
                    .collect::<Vec<_>>()
                    .join("+")
            );

            // TODO: add actually serving the compressed files... for now just do this!
            let link_files = &[link_files.first().unwrap()];

            for (file_path, url_suffix) in link_files {
                let embedded_file = EmbedAsset::get(file_path);

                if embedded_file.is_none() {
                    error!(
                        "Static file not found in embedded assets: {} (for url {})",
                        file_path,
                        urls.join(", ")
                    );
                    continue;
                }
                let embedded_file = embedded_file.unwrap();

                let urls = urls.iter().filter_map(|u| {
                    if u.ends_with(".css") || u.ends_with(".js") {
                        Some(format!("{}{}", u, url_suffix))
                    } else if url_suffix.is_empty() {
                        Some(u.to_string())
                    } else {
                        None
                    }
                });

                let body = match embedded_file.data {
                    std::borrow::Cow::Borrowed(slice) => Bytes::from_static(slice),
                    std::borrow::Cow::Owned(vec) => Bytes::from(vec),
                };
                let mime = mime_guess::from_path(file_path).first_or_octet_stream();
                let content_type = HeaderValue::from_str(mime.as_ref()).unwrap();

                for url in urls {
                    // trace!(url=%url, file_path=%file_path, "Url");
                    let body = body.clone();
                    let content_type = content_type.clone();

                    router = router.route(
                        &url,
                        get(move || async move {
                            ([(header::CONTENT_TYPE, content_type)], body).into_response()
                        }),
                    );
                }
            }
        }
    }

    Ok(router)
}
