use server::*;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    return start().await;
}
