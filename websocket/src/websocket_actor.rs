use act_zero::runtimes::tokio::spawn_actor;
use act_zero::*;
use act_zero_streams::StreamHandler;
use async_trait::async_trait;
use axum::extract::ws::{Message, WebSocket};

pub struct SimpleGreeter {
    number_of_greets: i32,
}

#[async_trait]
impl Actor for SimpleGreeter {
    async fn started(&mut self, _addr: Addr<Self>) -> ActorResult<()> {
        println!("Starting stream actor!");
        Produces::ok(())
    }
}

#[async_trait]
impl StreamHandler<Result<Message, axum::Error>> for SimpleGreeter {
    async fn started2(&mut self) {
        println!("Stream has started!");
    }

    async fn handle(&mut self, msg: Result<Message, axum::Error>) {
        if let Ok(msg) = msg {
            match msg {
                Message::Text(t) => {
                    println!("client send str: {:?}", t);
                    if let Ok(Produces::Value(greeting)) = self.greet(t).await {
                        println!("{}", greeting);
                    }
                }
                Message::Binary(_) => {
                    println!("client send binary data");
                }
                Message::Ping(_) => {
                    println!("socket ping");
                }
                Message::Pong(_) => {
                    println!("socket pong");
                }
                Message::Close(_) => {
                    println!("client disconnected");
                    return;
                }
            }
        }
    }

    async fn finished(&mut self) {
        println!("Stream has stopped!");
    }
}

impl SimpleGreeter {
    pub fn new(socket: WebSocket) {
        let actor = SimpleGreeter {
            number_of_greets: 0,
        };
        let addr = spawn_actor(actor);
        Self::add_stream(socket, addr.clone());
        let fut = addr.termination();
        tokio::spawn(async move {
            fut.await;
            println!("Actor has terminated!");
        });
    }

    async fn greet(&mut self, name: String) -> ActorResult<String> {
        self.number_of_greets += 1;
        Produces::ok(format!(
            "Hello, {}. You are number {}!",
            name, self.number_of_greets
        ))
    }
}
