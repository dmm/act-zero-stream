use std::error::Error;

use act_zero::*;
use act_zero_streams::StreamHandler;
use async_trait::async_trait;
use futures::executor::LocalPool;
use futures_util::stream::once;

struct SimpleGreeter {
    number_of_greets: i32,
}

#[async_trait]
impl Actor for SimpleGreeter {
    async fn started(&mut self, addr: Addr<Self>) -> ActorResult<()> {
        println!("Starting stream actor!");
        Self::add_stream(once(async { String::from("Mary") }), addr);
        Produces::ok(())
    }
}

#[async_trait]
impl StreamHandler<String> for SimpleGreeter {
    async fn started2(&mut self) {
        println!("Stream has started!");
    }

    async fn handle(&mut self, item: String) {
        println!("Got stream item: {}", item);
        if let Ok(Produces::Value(greeting)) = self.greet(item).await {
            println!("{}", greeting);
        }
    }

    async fn finished(&mut self) {
        println!("Stream has stopped!");
    }
}

impl SimpleGreeter {
    async fn greet(&mut self, name: String) -> ActorResult<String> {
        self.number_of_greets += 1;
        Produces::ok(format!(
            "Hello, {}. You are number {}!",
            name, self.number_of_greets
        ))
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut pool = LocalPool::new();
    let spawner = pool.spawner();
    pool.run_until(async move {
        let actor_ref = Addr::new(
            &spawner,
            SimpleGreeter {
                number_of_greets: 0,
            },
        )?;

        let greeting = call!(actor_ref.greet("John".into())).await?;
        println!("{}", greeting);

        let greeting = call!(actor_ref.greet("Emma".into())).await?;
        println!("{}", greeting);

        Ok(())
    })
}
