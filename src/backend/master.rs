use actix::prelude::*;
use actix::{Actor, StreamHandler};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use async_trait::async_trait;
use std::time::{Duration, Instant};

use crate::node::anvil::start_anvil;
use crate::node::proxy::init_proxy;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(0); // disable it for now

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// Define HTTP actor
pub struct MyWs {
    hb: Instant,
}

impl MyWs {
    pub fn new() -> Self {
        Self { hb: Instant::now() }
    }

    /// helper method that sends ping to client every 5 seconds (HEARTBEAT_INTERVAL).
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        let cringe = "
==============WELCOME SIR============
               °ooOOOoo°.               
             oO###OO###O°               
           .OOO#########.               
           *OO###o####OoO               
           oO###*o#####O#*              
           *OO##o##OO#####o             
           .OO##OO#O*..oO#*             
            oO######O*°                 
            .#O######Ooo°               
             °O#######O°@*              
             .O#########O°              
              .OOOOOOOO#.               
               °OOOOo*o°                
              °O########°               
            .°OOooooooooo°.             
           °OOOOOOOOOOoooOO°            
==============WELCOME SIR============
                        ";
        ctx.text(cringe);

        if HEARTBEAT_INTERVAL.as_secs() > 0 {
            self.hb(ctx);
        }
    }
}

/// Handler for ws::Message message
// #[async_trait]
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        /*match msg {
            Ok(ws::Message::Text(text)) => {
                dbg!(&text);

                let output = match text.to_string().as_str() {
                    "start" => {
                        init_proxy(9000, 9001);
                        ""
                    }
                    _ => "not recognized",
                };

                // ctx.text(output)

                //
            }
            _ => (),
        }*/

        let fut = async move {
            let status = init_proxy(9000, 9001)
                .await
                .expect("child process encountered an error");

            // println!("child status was: {}", status);
        };
        let fut = actix::fut::wrap_future::<_, Self>(fut);
        ctx.spawn(fut);
        // ctx.add_stream(reader.map(|l| Ok(Line(l.expect("Not a line")))));
    }
}

async fn index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(MyWs::new(), &req, stream);
    println!("{:?}", resp);
    resp
}

pub async fn start_server() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/chad/", web::get().to(index)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await?;

    Ok(())
}
