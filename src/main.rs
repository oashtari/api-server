use clap::{Parser, Subcommand};
use colored_json::prelude::*;
use hyper::{body::HttpBody as _, header::CONTENT_TYPE, Body, Client, Method, Request, Uri};
use router::create_router;
use serde_json::json;
use yansi::Paint;

mod api;
mod error;
mod router;
mod todo;

#[derive(Parser)]
struct Cli {
    /// Base URL of API service
    url: hyper::Uri,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List all todos
    List,
    /// Create a new todo
    Create {
        /// The todo body
        body: String,
    },
    /// Read a todo
    Read {
        /// The todo id
        id: i64,
    },
    /// Update a todo
    Update {
        /// The todo id
        id: i64,
        /// The todo body
        body: String,
        /// Mark todo as completed
        #[arg(short, long)]
        completed: bool,
    },
    /// Delete a todo
    Delete {
        /// The todo id
        id: i64,
    },
}

async fn request(
    url: hyper::Uri,
    method: Method,
    body: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();

    let mut res = client
        .request(
            Request::builder()
                .uri(url)
                .method(method)
                .header("Content-Type", "application/json")
                .body(
                    body.map(|s| Body::from(s))
                        .unwrap_or_else(|| Body::empty()),
                )?,
        )
        .await?;

    let mut buf = Vec::new();
    while let Some(next) = res.data().await {
        let chunk = next?;
        buf.extend_from_slice(&chunk);
    }
    let s = String::from_utf8(buf)?;

    eprintln!("Status: {}", Paint::green(res.status()));
    if res.headers().contains_key(CONTENT_TYPE) {
        let content_type = res.headers()[CONTENT_TYPE].to_str()?;
        eprintln!("Content-Type: {}", Paint::green(content_type));
        if content_type.starts_with("application/json") {
            println!("{}", &s.to_colored_json_auto()?);
        } else {
            println!("{}", &s);
        }
    } else {
        println!("{}", &s);
    }

    Ok(())
}
// async fn request(
//     url: hyper::Uri,
//     method: Method,
//     body: Option<String>,
// ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//     let client = Client::new();

//     let mut res = client
//         .request(
//             Request::builder()
//                 .uri(url)
//                 .method(method)
//                 .header("Content-Type", "application/json")
//                 .body(body.map(|s| Body::from(s)))
//                 .unwrap_or_else(|| Body::empty())?,
//         )
//         .await?;

//     let mut buf = Vec::new();

//     while let Some(next) = res.data().await {
//         let chunk = next?;
//         buf.extend_from_slice(&chunk);
//     }

//     let s = String::from_utf8(buf)?;

//     eprintln!("Status: {}", Paint::green(res.status()));
//     if res.headers().contains_key(CONTENT_TYPE) {
//         let content_type = res.headers()[CONTENT_TYPE].to_str()?;
//         eprintln!("Content-Type: {}", Paint::green(content_type));
//         if content_type.starts_with("application/json") {
//             println!("{}", &s.to_colored_json_auto()?);
//         } else {
//             println!("{}", &s);
//         }
//         else {
//             println!("{}", &s);
//         }
//         Ok(())
//     }

// }


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();

    let mut uri_builder = Uri::builder();

    if let Some(scheme) = cli.url.scheme() {
        uri_builder = uri_builder.scheme(scheme.clone());
    }

    if let Some(authority) = cli.url.authority() {
        uri_builder = uri_builder.authority(authority.cloen());
    }

    match cli.command {
        Commands::List => {
            request(
                uri_builder.path_and_query("/v1/todos").build()?;
            )
            .await
        }
        Commands::Delete { id } => {
            request(
                uri_builder.path_and_query(foamat!("/v1/todos/{}", id))
                .build()?,
                Method::DELETE,
                None,
            )
            .await
        }
        Commands::Read { id } => {
            request(
                uri_builder.path_and_query(format!("/v1/todos/{}", id)).build()?,
                Method::GET,
                None,
            )
            .await
        }
        Commands::Create { body } => {
            request(
                uri_builder.path_and_query("/v1/todos").build()?,
                Method::POST,
                Some(json!({ "body": body }).to_string()),
            )
            .await
        }
        Commands::Update { id, body, completed } => {
            request(
            uri_builder.path_and_query(format!("/v1/todos/{}", id)).build()?,
            Method::PUT,
            Some(json!({"body":body,"completed":completed}).to_string()),
        )
        .await
        }
    }
    // Initializes the tracing and logging for our service and its dependencies.
    init_tracing();

    // Initialized DB pool
    let dbpool = init_dbpool().await.expect("couldn't initialize db pool");

    // Creates the core application service and its routes.
    let router = create_router(dbpool).await;

    // Fetches the binding address from the environment variable BIND_ADDR, or uses the default value of 127.0.0.1:3000.
    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_string());

    // Parses the binding address into a socket address.
    // Creates the service and starts the HTTP server.
    axum::Server::bind(&bind_addr.parse().unwrap())
        .serve(router.into_make_service())
        .await
        .expect("unable to start server")
}

fn init_tracing() {
    use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*, EnvFilter};

    // Fetches the RUST_LOG environment variable, providing a default value if it’s not defined.

    let rust_log = std::env::var(EnvFilter::DEFAULT_ENV)
        .unwrap_or_else(|_| "sqlx=info,tower_http=debug,info".to_string());

    // Returns the default global registry.
    // Adds a formatting layer, which provides human-readable trace formatting.
    // Constructs an environment filter, with the default log level set to info, otherwise using the value provided by RUST_LOG.
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .parse_lossy(rust_log),
        )
        .init();
}

async fn init_dbpool() -> Result<sqlx::Pool<sqlx::Sqlite>, sqlx::Error> {
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use std::str::FromStr;

    // We’ll try to read the DATABASE_URL environment variable, or default to sqlite:db.sqlite
    // if not defined (which opens a file called db.sqlite in the current working directory)

    let db_connection_str =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:db.sqlite".to_string());

    // When we connect to the database, we ask the driver to create the database if it doesn’t already exist.
    let dbpool = SqlitePoolOptions::new()
        .connect_with(SqliteConnectOptions::from_str(&db_connection_str)?.create_if_missing(true))
        .await
        .expect("can't connect to database");

    // After we’ve connected to the DB, we run any migrations that are needed.
    // We can pass our newly created DB pool directly to SQLx, which will obtain a connection from the pool.
    sqlx::migrate!()
        .run(&dbpool)
        .await
        .expect("database migration failed.");

    Ok(dbpool)
}
