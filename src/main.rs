use std::boxed;
use std::str::FromStr;
use bytes::Bytes;
use futures::{future}; use http::HeaderValue;
use tokio;
use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Body, Request, Response, Uri, body};
use hyper::service::{make_service_fn, service_fn};
use hyper::client::{Client};
use clap::Parser;
use flate2::read::GzDecoder;
use std::io::Read;
use rand::distributions::{Alphanumeric, DistString};


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    target: Vec<String>,
}


fn replace_host(host: &str, original_uri: &Uri) -> Uri {
    let target_uri = Uri::from_str(host).unwrap();
    let mut uri_parts = original_uri.clone().into_parts();

    uri_parts.authority = target_uri.authority().cloned();
    uri_parts.scheme = target_uri.scheme().cloned();
    Uri::from_parts(uri_parts).unwrap()
}


fn build_request(host: &str, original: &hyper::http::request::Parts, body: &Bytes) -> hyper::Request<Body> {
    let mut to_forward = Request::builder()
        .method(original.method.clone())
        .uri(replace_host(host, &original.uri))
        .body(Body::from(body.clone()))
        .expect("Failed to clone request");
    *to_forward.headers_mut() = original.headers.clone();
    to_forward
}


fn get_reader<'a>(resp: &mut Response<Body>, body_reader: &'a [u8]) -> Box<dyn Read + 'a> {
    let boxed_body = Box::new(body_reader) as Box<dyn Read>;
    let encoding = resp.headers().get(hyper::header::CONTENT_ENCODING);

    encoding.map_or(boxed_body, |value| {
        match value.to_str().unwrap() {
            "gzip" => {
                Box::new(GzDecoder::new(&*body_reader)) as Box<dyn Read>
                }
            encoding => panic!("Error: unknown content-encoding header \"{encoding}\"")
        }
    })

}


async fn send_request(id: &String, client: &Client<hyper::client::HttpConnector>, host: &str, original: &hyper::http::request::Parts, body: &Bytes) -> Result<Result<Response<Body>, hyper::Error>, Result<Response<Body>, hyper::Error>> {
    println!("{id}: forwarding request to {host}");
    let request = build_request(host, original, body);

    println!("{id}: Sending request to {host}");
    let mut response_wrapped = client.request(request).await;
    println!("{id}: Received response from {host}");

    match response_wrapped.as_mut() {
        Err(_) => {
            println!("{id}: Host: {host}, {response_wrapped:?}");
            Err(response_wrapped)
        },
        Ok(resp) => {
            let bytes = hyper::body::to_bytes(resp.body_mut()).await.expect("failed to read response body as bytes");
            // Maybe use reqwest instead?
            // Initially the plan was not to parse the body, but to use HTTP codes for performance and reusability
            // Currently RPC returns HTTP200 even if it encounters an error though so it has to be parsed
            let json: serde_json::Value = serde_json::from_reader(get_reader(resp, &bytes)).expect("Unable to deserialize JSON body");

            println!("{id}: Host: {host}, {resp:?}, JSON: {json:?}");

            *(*resp).body_mut() = bytes.into();

            match json.get("error").is_none() {
                true => Ok(response_wrapped),
                false => Err(response_wrapped)
            }
        }

    }

}


async fn handle_request(original: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let id = Alphanumeric.sample_string(&mut rand::thread_rng(), 8);
    println!("{id}: {:?}", original);

    let client = Client::new();
    let urls = Args::parse().target;
    let (original_parts, body_stream) = original.into_parts();
    let body = hyper::body::to_bytes(body_stream).await.unwrap();

    let mut responses = future::join_all(urls.iter().map(|url| 
            send_request(&id, &client, url, &original_parts, &body)
        )).await;
    let default = responses.remove(0);
    let success = responses.into_iter().find(|result| result.is_ok() && result.as_ref().unwrap().is_ok());

    match success.or(Some(default)).unwrap() {
        Ok(response) => {
            println!("{id}: {:?}", response);
            response
        }
        Err(response) => {
            println!("{id}: All nodes returned an error. Sample response:{:?}", response);
            response
        }
    }
}


#[tokio::main]
async fn main() {
    // Parse arguments and display help if needed
    // FIXME: Parse arguments once and pass it to the handler
    Args::parse();
    const PORT: u16 = 8545;
    let addr = SocketAddr::from(([0, 0, 0, 0], PORT));

    let make_service = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(handle_request))
    });

    let server = hyper::server::Server::bind(&addr).serve(make_service);

    println!("Listening on http://{}", addr);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
