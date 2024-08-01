use fastly::http::{body, StatusCode};
use fastly::kv_store::KVStore;
use fastly::{Error, Request, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct StoredRequest {
    Method: String,
    Path: String,
    Host: String,
    Headers: HashMap<String, String>,
    Body: String,
}

#[fastly::main]
fn main(mut req: Request) -> Result<Response, Error> {
    println!(
        "FASTLY_SERVICE_VERSION: {}",
        std::env::var("FASTLY_SERVICE_VERSION").unwrap_or_else(|_| String::new())
    );

    // Pattern match on the path...
    match req.get_path() {
        // If request is to the `/` path...
        "/getrequests" => {
            let host = req
                .get_url()
                .host_str()
                .expect("Host must be configured")
                .to_string();
            let mut kvstore = KVStore::open("relay_store").unwrap().unwrap();
            let key = Some(kvstore.lookup(&host).unwrap());
            let key_value = match key.unwrap() {
                None => "[]",
                Some(k) => &k.into_string(),
            };
            let mut requests_buffer: Vec<StoredRequest> = serde_json::from_str(key_value).unwrap();

            if requests_buffer.is_empty() {
                Ok(Response::from_status(StatusCode::OK).with_body_text_plain(""))
            } else {
                let response = requests_buffer[0].clone();
                requests_buffer.remove(0);
                let mut body = body::Body::new();
                body.write_str(&serde_json::to_string(&requests_buffer).unwrap());
                kvstore.insert(&host, body);
                Ok(Response::from_status(StatusCode::OK)
                    .with_body_text_plain(&serde_json::to_string(&response).unwrap()))
            }
        }
        "/" => {
            let mut header_map = HashMap::new();
            let headers = req.get_headers();
            for (n, v) in headers {
                header_map.insert(n.as_str().to_owned(), v.to_str().unwrap().to_owned());
            }

            let request = StoredRequest {
                Method: req.get_method_str().to_string(),
                Host: req
                    .get_url()
                    .host_str()
                    .expect("Host must be configured")
                    .to_string(),
                Path: req.get_url().path().to_string(),
                Body: req.clone_with_body().into_body_str(),
                Headers: header_map,
            };

            // First get exisiting value.
            let mut kvstore = KVStore::open("relay_store").unwrap().unwrap();
            let key = Some(kvstore.lookup(&request.Host).unwrap());
            let key_value = match key.unwrap() {
                None => "[]",
                Some(k) => &k.into_string(),
            };
            let mut requests_buffer: Vec<StoredRequest> = serde_json::from_str(key_value).unwrap();

            //Add new request to request_buffer to KV.
            requests_buffer.push(request.clone());

            // Write new request_buffer to KV.
            let mut body = body::Body::new();
            body.write_str(&serde_json::to_string(&requests_buffer).unwrap());
            kvstore.insert(&request.Host, body);

            Ok(Response::from_status(StatusCode::OK).with_body_text_plain(""))
        }

        // Catch all other requests and return a 404.
        _ => Ok(Response::from_status(StatusCode::NOT_FOUND)
            .with_body_text_plain("The page you requested could not be found\n")),
    }
}
