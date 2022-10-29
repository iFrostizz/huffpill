// Simple proxy that forwards all the requests to anvil, and filters cheating requests

use std::future::{Future, Ready};

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
use awc::Client;
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
            todo!();
        }
    };

    dbg!(&json_rpc_req);

    // if json_rpc_req.method.method

    // dbg!(&json_rpc_req);

    let mut new_url = url.get_ref().clone();
    new_url.set_path(req.uri().path());
    new_url.set_query(req.uri().query());

    // TODO: This forwarded implementation is incomplete as it only handles the unofficial
    // X-Forwarded-For header but not the official Forwarded one.
    let forwarded_req = client
        .request_from(new_url.as_str(), req.head())
        .no_decompress();
    /*let forwarded_req = match req.head().peer_addr {
        Some(addr) => forwarded_req.insert_header(("x-forwarded-for", format!("{}", addr.ip()))),
        None => forwarded_req,
    };*/

    dbg!(&forwarded_req);

    /*let res = forwarded_req
    .send_stream(bytes)
    .await
    .map_err(error::ErrorInternalServerError)?;*/

    // let mut client_resp = HttpResponse::build(bytes);
    // Remove `Connection` as per
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection#Directives
    /*for (header_name, header_value) in res.headers().iter().filter(|(h, _)| *h != "connection") {
        client_resp.insert_header((header_name.clone(), header_value.clone()));
    }*/

    let res = forwarded_req
        .send_body(bytes)
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

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Client::default()))
            .app_data(web::Data::new(anvil_endpoint.clone()))
            .wrap(middleware::Logger::default())
            .default_service(web::to(forward))
    })
    .bind((forward_url.host_str().unwrap(), forward_url.port().unwrap()))?
    .workers(2)
    .run()
    .await
}
