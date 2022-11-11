// Simple proxy that forwards all the requests to anvil, and filters cheating requests

use std::{fs::File, io::prelude::*};

use crate::{
    backend::convention::{Request, Response},
    node::anvil::start_anvil,
};
// use actix::prelude::Stream;
use actix_web::{
    error,
    http::StatusCode,
    middleware,
    web::{self, BytesMut},
    App, Error, FromRequest, HttpRequest, HttpResponse, HttpServer,
};
use awc::{Client, ClientRequest};
use serde::Serialize;
use url::Url;

use futures_core::stream::Stream;
use futures_util::StreamExt;

#[derive(Serialize, Debug)]
struct Info {
    name: String,
}

pub async fn forward(
    req: HttpRequest,
    mut payload: web::Payload,
    url: web::Data<Url>,
    client: web::Data<Client>,
    forbidden_methods: web::Data<Vec<String>>,
) -> Result<HttpResponse, Error> {
    let size = payload.size_hint();
    if size.0 > 1000 {
        return Ok(HttpResponse::new(StatusCode::PAYLOAD_TOO_LARGE));
    }

    let mut bytes = web::BytesMut::new();

    while let Some(item) = payload.next().await {
        bytes.extend_from_slice(&item?);
    }

    let json_rpc_req: Request = match serde_json::from_slice(bytes.as_ref()) {
        Ok(ok) => ok,
        Err(err) => {
            dbg!(&err);
            return Ok(HttpResponse::new(StatusCode::IM_A_TEAPOT));
        }
    };

    if forbidden_methods.contains(&json_rpc_req.method) {
        return Ok(HttpResponse::new(StatusCode::METHOD_NOT_ALLOWED));
    }

    let mut new_url = url.get_ref().clone();
    new_url.set_path(req.uri().path());
    new_url.set_query(req.uri().query());

    let forwarded_req = client
        .request_from(new_url.as_str(), req.head())
        .no_decompress();

    dbg!(&forwarded_req);

    build_and_stream_res(forwarded_req, bytes).await
}

pub async fn build_and_stream_res(
    req: ClientRequest,
    body: BytesMut,
) -> Result<HttpResponse, Error> {
    let res = req
        .send_body(body)
        .await
        .map_err(error::ErrorInternalServerError)?;
    let mut client_resp = HttpResponse::build(res.status());

    Ok(client_resp.streaming(res))
}

pub async fn init_proxy(in_port: u16, out_port: u16) -> std::io::Result<()> {
    let anvil = start_anvil(in_port, 30 * 60);
    println!("Anvil running at `{}`", anvil.endpoint());

    dbg!(&anvil.endpoint());

    let anvil_endpoint = Url::parse(&anvil.endpoint()).unwrap();

    let forward_url = format!("http://127.0.0.1:{out_port}");
    let forward_url = Url::parse(&forward_url).unwrap();

    let mut file = File::open("reference.txt").unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    let forbidden_methods = content
        .split('\n')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Client::default()))
            .app_data(web::Data::new(anvil_endpoint.clone()))
            .app_data(web::Data::new(forbidden_methods.clone()))
            .wrap(middleware::Logger::default())
            .default_service(web::to(forward))
    })
    .bind((forward_url.host_str().unwrap(), forward_url.port().unwrap()))?
    .workers(2)
    .run()
    .await
}
