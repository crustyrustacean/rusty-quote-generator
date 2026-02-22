// src/startup.rs

// dependencies
use crate::configuration::Settings;
use crate::routes::health_check;
use actix_files::Files;
use actix_web::dev::Server;
use actix_web::{App, HttpServer, web, web::Data};
use std::net::TcpListener;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr()?.port();
        let server = run(listener, configuration.application.base_url).await?;
        Ok(Self { port, server })
    }

    #[allow(dead_code)]
    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub struct ApplicationBaseUrl(pub String);

async fn run(listener: TcpListener, base_url: String) -> Result<Server, anyhow::Error> {
    let base_url = Data::new(ApplicationBaseUrl(base_url));
    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .service(Files::new("/", "../public").index_file("index.html").prefer_utf8(true))
            .app_data(base_url.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
