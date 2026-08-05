# CONTRIBUTING

## LLM Skill

**Rust Core Directives**

1. **Clean & Agnostic Code:** Write logically grouped, modular code. While the logic may be specific, the design should follow clean-code principles that make it robust and context-independent where possible.

2. **Crystal Clear Readability:** Prioritize maintainability. Code must be self-explanatory.

3. **Mandatory Documentation:** Every struct, field, enum, and function must have standard Rustdoc comments (`///`) explaining *what* it does. Assume that from every crate a HTML docs will be built. Maintain same level of quality as popular open source libraries on docs.rs. Besides rustdoc, use normal comments inside code blocks to explain each logical place shortly, so you can quickly orientate in code. Example:

```rust
/// Protects the buffer using a CRC checksum, which is appended to the end of the buffer. A new buffer len is returned.
fn protect(&self, buf: &mut [u8], header_len: usize, payload_len: usize) -> Result<usize, PacketError> {
    
    // Compute checksum over the payload
    let total_len = header_len + payload_len;
    let mut hasher = Hasher::new();
    hasher.update(&buf[..total_len]);
    let checksum = hasher.finalize();
    
    // Append the checksum to the buffer
    buf[total_len..total_len+4].copy_from_slice(&checksum.to_be_bytes());

    // Return the new packet size
    Ok(total_len + 4)
}
```

4. **Strict Error Handling:** Never use generic `Box<dyn Error>` or `String` for errors. You must use the `thiserror` crate to define explicit, context-rich error enums for all fail states. Alternatively, you can also use `anyhow`. The use of plain `unwrap` that panics is not permitted (exception are the infallible ones). If there is a place where you can use `unwrap` safely, use `expect` instead with a short message.

5. **Unified Logging:** Use the `scaffolding_logs` crate for all instrumentation. It contains wrapper macros around `tracing`. Color guide:
- Use `MAGENTA` for starting a new service/workers/listeners/etc.
- Use `GREEN` for main function tasks.
- Use `YELLOW` for background tasks.
- Use `BLUE` for Axum API handlers.
- Use `RED` for critical errors that MUST get attention and `herror` is not enough.

Example:

```rust
use scaffolding_logs::*;
// Prints: "[SERVICE]: Starting project-0.0.1"
hinfo!(MAGENTA, "SERVICE", "Starting {name}-{version}");
// Other variants are awailable
hwarn!(YELLOW, "FILES", "Unknown warning!");
herror!(RED, "HTML", "Frontend failed to render");
```

6. **Clippy Compliance:** Code must pass clippy lints. Fix logic errors proactively.

7. **The "Never Nesting" Rule:** Absolutely no deep nesting (arrow code). Use early returns, the `?` operator, guard clauses, and aggressively extract inner blocks into separate, logically named helper functions.

8. **Formatting:** Your code MUST copy the existing codestyle of the repository. You must ensure the format is COPIED without error or your changes will get rejected. Never change spacing, tabs, new lines on existing code. Running any FORMATTING IS STRICTLY FORBIDDED. If you do so, your entire work will get rejected during PR review.

Example of a good formatted Rust code with comments. You must copy this style:

```rust
#[tokio::main]
async fn main() -> Result<(), String> {
    // Initiate the logs and enviroment variables
    dotenvy::dotenv().ok(); tracing_init("info,sqlx=warn"); println!();

    // Print service info to the console
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    hinfo!(MAGENTA, "SERVICE", "Starting {name}-{version}");

    // Create a cancellation token for graceful async shutdown (Ctrl+C)
    let cancel_token = shutdown_signal();

    // Start the service and inspect the result
    tokio::select! {
        value = start_service() => value,
        _ = cancel_token.cancelled() => Ok(()),
    }
        .map(|_| hinfo!(GREEN, "EXIT", "Server closed"))
        .map_err(|e| {herror!(RED, "CRASH", "{e}"); e.to_string()})
}


// #=====================#
// #=== SERVICE ENTRY ===#

/// Service configuration
#[derive(Clone, Debug)]
pub struct ServiceConfig {
    /// Listener bound socket address
    pub host_adress: SocketAddr,
    /// The JWT secret token
    pub jwt_secret_token: String,
    /// Database configuration
    pub database: DatabaseConfig,
} // (Note no new line here)
impl ServiceConfig {
    /// Creates new config from environment variables
    pub fn from_env() -> anyhow::Result<Self> {

        // Sample the env
        let host_adress = env::var("HOST_ADDRESS").unwrap_or("0.0.0.0:8888".into()).parse()?;
        let jwt_secret_token = env::var("JWT_SECRET").unwrap_or(String::from("my-secret-token"));
        
        // Return the struct
        Ok(Self {
            host_adress,
            jwt_secret_token,
            database: DatabaseConfig::from_env()?,
        })
    }
}

/// Starts the service
pub async fn start_service() -> anyhow::Result<()> {
    // Load configuration.
    let config = config::ServiceConfig::from_env().map_err(|e| anyhow!("Configuration error: {e}"))?;

    // Connect to the database.
    let database = config.database.get_client().await.map_err(|e| anyhow!("Database error: {e}"))?;
    hinfo!(YELLOW, "DATABASE", "Connected to Database");

    // Run idempotent schema migrations.
    database::migration::migrate(&database).await.map_err(|e| anyhow!("Migration error: {e}"))?;
    hinfo!(YELLOW, "DATABASE", "Schema migrated");

    // Seed an admin user from env vars if none exists yet.
    match database::seed::seed_admin(&database).await {
        Ok(Some(name)) => hwarn!(YELLOW, "ADMIN", "New Admin created: '{name}'"),
        Ok(None) => hinfo!(GREEN, "ADMIN", "No admin created"),
        Err(e) => herror!(RED, "ADMIN", "Seed failed: {e}"),
    }

    // Load all small files in the public directory
    let public = ArcSwap::new(Arc::new(files::MemoryFileServer::new("./assets/public/", Some(MEGABYTE), true).await.map_err(|e| anyhow!("File error: {e}"))?));
    hinfo!(YELLOW, "FILES", "Public files loaded");

    // Create the controller state.
    let state = Arc::new(Controller {
        public,
        database,
        config,
    });

    // Spawn background worker
    let worker_state = Arc::clone(&state);
    tokio::spawn(async move {
        background_public_cache_updater(worker_state).await;
    });

    // Notify frontend worker
    state.frontend_notify.notify_one();

    // Mount the API onto a TCP server.
    hinfo!(GREEN, "SERVICE", "Listening on http://{}", state.config.host_adress);
    let listener = TcpListener::bind(state.config.host_adress).await?;

    // Wrap the router with the normalization layer
    let app = ServiceBuilder::new()
        .layer(NormalizePathLayer::trim_trailing_slash())
        .service(api::api(state).layer(CompressionLayer::new()));

    // Serve the layered application
    axum::serve(listener, tower::make::Shared::new(app)).await?;

    Ok(())
}
```

9. **Idiomatic Rust:** You must use idiomatic Rust. Reuse standard library where possible, aggresively use chaining of functions to modify data (functional programming). Do not create unnecessary functions if you can achieve the same result with a few chained functions.

10. **Git:** Touching git is STRICTLY FORBIDDEN. You are NOT ALLOWED ANY GIT MANIPULATION.
