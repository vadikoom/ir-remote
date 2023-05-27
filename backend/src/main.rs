#[macro_use]
extern crate rocket;

#[get("/hello")]
fn hello() -> &'static str {
    "Hello, world!"
}

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    rocket::build().mount("/", routes![hello]).launch().await?;
    Ok(())
}