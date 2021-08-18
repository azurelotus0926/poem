use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use askama::Template;
use tokio::fs::File;

use crate::{
    error::ErrorInternalServerError,
    http::{Method, StatusCode},
    Body, Endpoint, Error, Request, Response, Result,
};

#[derive(Template)]
#[template(
    ext = "html",
    source = r#"
<html>
    <head>
        <title>Index of {{ path }}</title>
    </head>
    <body>
        <h1>Index of {{ path }}</h1>
        <ul>
            {% for file in files %}
            <li>
                {% if file.is_dir %} 
                <a href="{{ file.url }}">{{ file.filename | e }}/</a>
                {% else %}
                <a href="{{ file.url }}">{{ file.filename | e }}</a>
                {% endif %}
            </li>
            {% endfor %}
        </ul>
    </body>
    </html>
"#
)]
struct DirectoryTemplate<'a> {
    path: &'a str,
    files: Vec<FileRef>,
}

struct FileRef {
    url: String,
    filename: String,
    is_dir: bool,
}

/// Static files handling service.
pub struct Files {
    path: PathBuf,
    show_files_listing: bool,
    index_file: Option<String>,
}

impl Files {
    /// Create new Files service for a specified base directory.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            show_files_listing: false,
            index_file: None,
        }
    }

    /// Show files listing for directories.
    ///
    /// By default show files listing is disabled.
    pub fn show_files_listing(self) -> Self {
        Self {
            show_files_listing: true,
            ..self
        }
    }

    /// Set index file
    ///
    /// Shows specific index file for directories instead of showing files
    /// listing.
    ///
    /// If the index file is not found, files listing is shown as a fallback if
    /// Files::show_files_listing() is set.
    pub fn index_file(self, index: impl Into<String>) -> Self {
        Self {
            index_file: Some(index.into()),
            ..self
        }
    }
}

#[async_trait::async_trait]
impl Endpoint for Files {
    async fn call(&self, req: Request) -> Result<Response> {
        if req.method() != Method::GET {
            return Err(Error::status(StatusCode::METHOD_NOT_ALLOWED));
        }

        let path = req.uri().path();
        let path = path.trim_start_matches('/');
        let path = path.trim_end_matches('/');
        let mut file_path = self.path.clone();
        for p in Path::new(path) {
            if p == OsStr::new(".") {
                continue;
            } else if p == OsStr::new("..") {
                file_path.pop();
            } else {
                file_path.push(&p);
            }
        }

        if !file_path.starts_with(&self.path) {
            return Err(Error::status(StatusCode::FORBIDDEN));
        }

        if !file_path.exists() {
            return Err(Error::status(StatusCode::NOT_FOUND));
        }

        if file_path.is_file() {
            create_file_response(&file_path).await
        } else {
            if let Some(index_file) = &self.index_file {
                let index_path = file_path.join(index_file);
                if index_path.is_file() {
                    return create_file_response(&index_path).await;
                }
            }

            if self.show_files_listing {
                let read_dir = file_path
                    .read_dir()
                    .map_err(|_| Error::status(StatusCode::INTERNAL_SERVER_ERROR))?;
                let mut template = DirectoryTemplate {
                    path: req.uri().path(),
                    files: Vec::new(),
                };

                for res in read_dir {
                    let entry =
                        res.map_err(|_| Error::status(StatusCode::INTERNAL_SERVER_ERROR))?;
                    if let Some(filename) = entry.file_name().to_str() {
                        let mut base_url = req.original_uri().path().to_string();
                        if !base_url.ends_with('/') {
                            base_url.push('/');
                        }
                        template.files.push(FileRef {
                            url: format!("{}{}", base_url, filename),
                            filename: filename.to_string(),
                            is_dir: entry.path().is_dir(),
                        });
                    }
                }

                let html = template
                    .render()
                    .map_err(|_| Error::status(StatusCode::INTERNAL_SERVER_ERROR))?;
                Ok(Response::builder().body(Body::from_string(html)))
            } else {
                Err(Error::status(StatusCode::NOT_FOUND))
            }
        }
    }
}

async fn create_file_response(path: &Path) -> Result<Response> {
    let file = File::open(path)
        .await
        .map_err(ErrorInternalServerError::new)?;
    Ok(Response::builder().body(Body::from_async_read(file)))
}
