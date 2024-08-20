use chaching_clan::bot::run_dispatcher;

#[tokio::main]
pub async fn main() {
    pretty_env_logger::init();
    dotenv::dotenv().ok();

    let bot = teloxide::Bot::from_env();

    run_dispatcher(bot).await;
}
