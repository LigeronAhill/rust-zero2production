use zero2prod::run;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    run(3000)?.await
}
