use bgm::task::exec;

#[tokio::main]
async fn main() {
    exec().await.unwrap();
}
