pub mod auth_routes;
pub mod device_routes;
pub mod handlers;

#[cfg(feature = "web-api")]
pub async fn start_server(port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use std::net::SocketAddr;
    
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    log::info!("Server listening on {}", addr);
    
    // This function is now essentially a stub, as we handle the server creation
    // directly in main.rs's start_web_server function
    log::info!("Server configuration moved to start_web_server function in main.rs");
    
    Ok(())
}
