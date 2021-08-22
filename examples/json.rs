use poem::{post, route, web::Json, EndpointExt, IntoResponse, Server};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct JsonType1 {
    name: String,
}

#[post]
fn hello(Json(json1): Json<JsonType1>) -> String {
    format!(r#"{{"code": 0, "message": "{}"}}"#, json1.name)
}

// right:
// curl -d '{"name": "Jack"}' http://127.0.0.1:3000/hello
// {"code": 0, "message": "hello: Jack"}
//
// error:
// curl -d '{"badkey": "Jack"}' http://127.0.0.1:3000/hello
// {"code": 1, "message": "missing field `name` at line 1 column 20"}
#[tokio::main]
async fn main() {
    let app = route().at("/hello", hello).after(|mut resp| async move {
        if !resp.status().is_success() {
            // returns the custom json error
            let reason = resp.take_body().into_string().await?;
            return Ok(Json(serde_json::json!( {
                "code": 1,
                "message": reason,
            }))
            .into_response());
        }
        Ok(resp)
    });
    let server = Server::bind("127.0.0.1:3000").await.unwrap();
    server.run(app).await.unwrap();
}
