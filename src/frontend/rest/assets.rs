//! Serves static files from a asset directory.

extern crate mime_guess;

use self::mime_guess::{get_mime_type, octet_stream};

macro_rules! include_files_as_assets {
    ( $target_match:expr, $( $file_name:expr ),* ) => {
        match $target_match {
            $(
                $file_name => Some(include_bytes!(concat!(concat!(env!("OUT_DIR"), "/static/"), $file_name)).as_ref()),
            )*
            _ => None
        }
    }
}

/// Returns a static file based upon a given String as a Path.
///
/// file_path: String path, beginning with a /
pub fn file_from_string(file_path: &str) -> Option<(String, &'static [u8])> {
    let guessed_mime = match file_path.rfind('.') {
        Some(ext_ptr) => {
            let ext = &file_path[ext_ptr + 1..];

            get_mime_type(ext)
        }
        None => octet_stream(),
    };

    let string_mime = guessed_mime.to_string();

    let contents = include_files_as_assets!(
        file_path,
        "/index.html",
        "/favicon.ico",
        "/img/logo.png",
        "/img/how-to-open.png",
        "/css/app.css",
        "/css/chunk-vendors.css",
        "/fonts/roboto-v18-latin-regular.eot",
        "/fonts/roboto-v18-latin-regular.woff",
        "/fonts/roboto-v18-latin-regular.woff2",
        "/js/chunk-vendors.js",
        "/js/chunk-vendors.js.map",
        "/js/app.js",
        "/js/app.js.map"
    )?;

    Some((string_mime, contents))
}
