use std::collections::HashMap;

use poem::{listener::TcpListener, route, Error, Result};
use poem_openapi::{
    payload::{Binary, Json},
    types::multipart::Upload,
    Multipart, Object, OpenApi, OpenApiService, Response,
};
use tokio::sync::Mutex;

#[derive(Debug, Object, Clone)]
struct File {
    name: String,
    desc: Option<String>,
    content_type: Option<String>,
    filename: Option<String>,
    data: Vec<u8>,
}

#[derive(Debug, Response)]
enum GetFileResponse {
    #[oai(status = 200)]
    Ok(Binary, #[oai(header = "Content-Disposition")] String),
    /// File not found
    #[oai(status = 404)]
    NotFound,
}

struct Status {
    id: u64,
    files: HashMap<u64, File>,
}

#[derive(Debug, Multipart)]
struct UploadPayload {
    name: String,
    desc: Option<String>,
    file: Upload,
}

struct Api {
    status: Mutex<Status>,
}

#[OpenApi]
impl Api {
    /// Upload file
    #[oai(path = "/files", method = "post")]
    async fn upload(&self, upload: UploadPayload) -> Result<Json<u64>> {
        let mut status = self.status.lock().await;
        let id = status.id;
        status.id += 1;

        let file = File {
            name: upload.name,
            desc: upload.desc,
            content_type: upload.file.content_type().map(ToString::to_string),
            filename: upload.file.file_name().map(ToString::to_string),
            data: upload.file.into_vec().await.map_err(Error::bad_request)?,
        };
        status.files.insert(id, file);
        Ok(Json(id))
    }

    /// Get file
    #[oai(path = "/files/:id", method = "get")]
    async fn get(&self, #[oai(name = "id", in = "path")] id: u64) -> GetFileResponse {
        let status = self.status.lock().await;
        match status.files.get(&id) {
            Some(file) => {
                let mut content_disposition = String::from("attachment");
                if let Some(file_name) = &file.filename {
                    content_disposition += &format!("; filename={}", file_name);
                }
                GetFileResponse::Ok(file.data.clone().into(), content_disposition)
            }
            None => GetFileResponse::NotFound,
        }
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:3000");
    let api_service = OpenApiService::new(Api {
        status: Mutex::new(Status {
            id: 1,
            files: Default::default(),
        }),
    })
    .title("Upload Files")
    .server("http://localhost:3000/api");
    let ui = api_service.swagger_ui("http://localhost:3000");

    poem::Server::new(listener)
        .await
        .unwrap()
        .run(route().nest("/api", api_service).nest("/", ui))
        .await
        .unwrap();
}
