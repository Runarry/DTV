use actix_web::{dev::ServerHandle, web, App, HttpResponse, HttpServer, Responder};
use futures_util::TryStreamExt;
use reqwest::Client;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::TcpStream;
use std::sync::Mutex as StdMutex;
use std::time::Duration;
use tauri::{AppHandle, State};

// Define a struct to hold the server handle in a Tauri managed state
#[derive(Default)]
pub struct ProxyServerHandle(pub StdMutex<Option<ServerHandle>>);

struct FlvProxySession {
    handle: ServerHandle,
    port: u16,
    upstream_url: String,
    platform: String,
    room_id: Option<String>,
}

#[derive(Default)]
pub struct FlvProxySessionManager(pub StdMutex<HashMap<String, FlvProxySession>>);

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartFlvProxySessionPayload {
    upstream_url: String,
    platform: String,
    room_id: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartFlvProxySessionResponse {
    session_id: String,
    proxy_url: String,
}

async fn find_free_port() -> u16 {
    // Using a fixed port as requested by the user for easier debugging
    34719
}

fn build_proxy_http_client() -> Client {
    Client::builder()
        .no_proxy()
        .http1_only()
        .gzip(false)
        .brotli(false)
        .no_deflate()
        .pool_idle_timeout(None)
        .pool_max_idle_per_host(4)
        .tcp_keepalive(Duration::from_secs(60))
        .timeout(Duration::from_secs(7200))
        .build()
        .expect("failed to build client")
}

fn generate_session_id() -> String {
    use rand::RngCore;

    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

#[derive(Deserialize)]
struct ImageQuery {
    url: String,
}

async fn image_proxy_handler(
    query: web::Query<ImageQuery>,
    client: web::Data<Client>,
) -> impl Responder {
    let url = query.url.clone();
    if url.is_empty() {
        return HttpResponse::BadRequest().body("Missing url query parameter");
    }

    let mut req = client
        .get(&url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        )
        .header(
            "Accept",
            "image/avif,image/webp,image/apng,image/*;q=0.8,*/*;q=0.5",
        );

    // Set a Referer to bypass hotlink protections
    if url.contains("hdslb.com") || url.contains("bilibili.com") {
        req = req
            .header("Referer", "https://live.bilibili.com/")
            .header("Origin", "https://live.bilibili.com");
    } else if url.contains("huya.com") {
        req = req
            .header("Referer", "https://www.huya.com/")
            .header("Origin", "https://www.huya.com");
    } else if url.contains("douyin") || url.contains("douyinpic.com") {
        req = req.header("Referer", "https://www.douyin.com/");
    }

    match req.send().await {
        Ok(upstream_response) => {
            let content_type = upstream_response
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("application/octet-stream")
                .to_string();

            // 为避免 Windows 下 chunked 传输的 Early-EOF，改为一次性读取 bytes 并返回
            if upstream_response.status().is_success() {
                match upstream_response.bytes().await {
                    Ok(bytes) => HttpResponse::Ok()
                        .content_type(content_type)
                        .insert_header(("Content-Length", bytes.len().to_string()))
                        .insert_header(("Cache-Control", "no-store"))
                        .body(bytes),
                    Err(e) => {
                        eprintln!("[Rust/proxy.rs image] Failed to read bytes: {}", e);
                        HttpResponse::InternalServerError()
                            .body(format!("Failed to read image bytes: {}", e))
                    }
                }
            } else {
                let status_from_reqwest = upstream_response.status();
                let error_text = upstream_response
                    .text()
                    .await
                    .unwrap_or_else(|e| format!("Failed to read error body from upstream: {}", e));
                eprintln!(
                    "[Rust/proxy.rs image] Upstream request to {} failed with status: {}. Body: {}",
                    url, status_from_reqwest, error_text
                );
                let actix_status_code =
                    actix_web::http::StatusCode::from_u16(status_from_reqwest.as_u16())
                        .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);

                HttpResponse::build(actix_status_code).body(format!(
                    "Error fetching IMAGE from upstream (reqwest): {}. Status: {}. Details: {}",
                    url, status_from_reqwest, error_text
                ))
            }
        }
        Err(e) => {
            eprintln!(
                "[Rust/proxy.rs image] Failed to send request to upstream {}: {}",
                url, e
            );
            HttpResponse::InternalServerError()
                .body(format!("Error connecting to upstream IMAGE {}: {}", url, e))
        }
    }
}

#[derive(Deserialize)]
struct FlvQuery {
    url: String,
}

async fn proxy_flv_stream(client: &Client, url: String) -> HttpResponse {
    if url.is_empty() {
        return HttpResponse::BadRequest().body("Missing upstream url");
    }

    println!(
        "[Rust/proxy.rs handler] Incoming FLV proxy request -> {}",
        url
    );

    let mut req = client
        .get(&url.clone())
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        )
        .header("Accept", "video/x-flv,application/octet-stream,*/*")
        .header("Range", "bytes=0-")
        .header("Connection", "keep-alive");

    // 如果是虎牙域名，添加必要的 Referer/Origin 头
    if url.contains("huya.com") || url.contains("hy-cdn.com") || url.contains("huyaimg.com") {
        req = req
            .header("Referer", "https://www.huya.com/")
            .header("Origin", "https://www.huya.com");
    }
    // 如果是B站域名，添加必要的 Referer 头
    if url.contains("bilivideo") || url.contains("bilibili.com") || url.contains("hdslb.com") {
        req = req.header("Referer", "https://live.bilibili.com/");
    }

    match req.send().await {
        Ok(upstream_response) => {
            if upstream_response.status().is_success() {
                let mut response_builder = HttpResponse::Ok();
                response_builder
                    .content_type("video/x-flv")
                    .insert_header(("Connection", "keep-alive"))
                    .insert_header(("Cache-Control", "no-store"))
                    .insert_header(("Accept-Ranges", "bytes"));

                let byte_stream = upstream_response.bytes_stream().map_err(|e| {
                    eprintln!(
                        "[Rust/proxy.rs handler] Error reading bytes from upstream: {}",
                        e
                    );
                    actix_web::error::ErrorInternalServerError(format!(
                        "Upstream stream error: {}",
                        e
                    ))
                });

                response_builder.streaming(byte_stream)
            } else {
                let status_from_reqwest = upstream_response.status(); // Renamed for clarity
                let error_text = upstream_response
                    .text()
                    .await
                    .unwrap_or_else(|e| format!("Failed to read error body from upstream: {}", e));
                eprintln!(
                    "[Rust/proxy.rs handler] Upstream request to {} failed with status: {}. Body: {}",
                    url, status_from_reqwest, error_text
                );
                // Convert reqwest::StatusCode to actix_web::http::StatusCode
                let actix_status_code =
                    actix_web::http::StatusCode::from_u16(status_from_reqwest.as_u16())
                        .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);

                HttpResponse::build(actix_status_code).body(format!(
                    "Error fetching FLV stream from upstream (reqwest): {}. Status: {}. Details: {}",
                    url, status_from_reqwest, error_text
                ))
            }
        }
        Err(e) => {
            eprintln!(
                "[Rust/proxy.rs handler] Failed to send request to upstream {} with reqwest: {}",
                url, e
            );
            return HttpResponse::InternalServerError().body(format!(
                "Error connecting to upstream FLV stream {} with reqwest: {}",
                url, e
            ));
        }
    }
}

// Legacy query-based FLV proxy
async fn flv_proxy_handler(
    query: web::Query<FlvQuery>,
    client: web::Data<Client>,
) -> impl Responder {
    proxy_flv_stream(client.get_ref(), query.url.clone()).await
}

// Session-based FLV proxy with fixed upstream URL
async fn flv_proxy_session_handler(
    upstream_url: web::Data<String>,
    client: web::Data<Client>,
) -> impl Responder {
    proxy_flv_stream(client.get_ref(), upstream_url.get_ref().clone()).await
}

#[tauri::command]
pub async fn start_flv_proxy_session(
    _app_handle: AppHandle,
    session_manager: State<'_, FlvProxySessionManager>,
    payload: StartFlvProxySessionPayload,
) -> Result<StartFlvProxySessionResponse, String> {
    let upstream_url = payload.upstream_url.trim().to_string();
    if upstream_url.is_empty() {
        return Err("upstream_url is required".to_string());
    }

    let session_id = generate_session_id();
    let runtime_session_id = session_id.clone();
    let runtime_platform = payload.platform.clone();
    let runtime_room_id = payload.room_id.clone();
    let app_data_upstream_url = web::Data::new(upstream_url.clone());

    let server_builder = HttpServer::new(move || {
        let app_data_reqwest_client = web::Data::new(build_proxy_http_client());
        App::new()
            .app_data(app_data_reqwest_client)
            .app_data(app_data_upstream_url.clone())
            .wrap(actix_cors::Cors::permissive())
            .route("/live.flv", web::get().to(flv_proxy_session_handler))
    })
    .keep_alive(Duration::from_secs(120))
    .bind(("127.0.0.1", 0))
    .map_err(|e| format!("[Rust/proxy.rs] Failed to bind FLV session proxy: {}", e))?;

    let port = server_builder
        .addrs()
        .first()
        .map(|addr| addr.port())
        .ok_or_else(|| {
            "[Rust/proxy.rs] Failed to resolve bound address for FLV session".to_string()
        })?;
    let runtime_port = port;

    let server = server_builder.run();
    let handle = server.handle();

    {
        let mut guard = session_manager.0.lock().unwrap();
        guard.insert(
            session_id.clone(),
            FlvProxySession {
                handle,
                port,
                upstream_url: upstream_url.clone(),
                platform: payload.platform.clone(),
                room_id: payload.room_id.clone(),
            },
        );
    }

    tauri::async_runtime::spawn(async move {
        if let Err(e) = server.await {
            eprintln!(
                "[Rust/proxy.rs] FLV session {} (platform={}, room={:?}, port={}) run error: {}",
                runtime_session_id, runtime_platform, runtime_room_id, runtime_port, e
            );
        } else {
            println!(
                "[Rust/proxy.rs] FLV session {} (platform={}, room={:?}) on port {} shut down.",
                runtime_session_id, runtime_platform, runtime_room_id, runtime_port
            );
        }
    });

    Ok(StartFlvProxySessionResponse {
        session_id,
        proxy_url: format!("http://127.0.0.1:{}/live.flv", port),
    })
}

#[tauri::command]
pub async fn stop_flv_proxy_session(
    session_manager: State<'_, FlvProxySessionManager>,
    session_id: String,
) -> Result<(), String> {
    let session = {
        let mut guard = session_manager.0.lock().unwrap();
        guard.remove(&session_id)
    };

    if let Some(session) = session {
        println!(
            "[Rust/proxy.rs] stop_flv_proxy_session: session={} platform={} room={:?} port={} upstream={}",
            session_id, session.platform, session.room_id, session.port, session.upstream_url
        );
        session.handle.stop(false).await;
    } else {
        println!(
            "[Rust/proxy.rs] stop_flv_proxy_session: session {} not found (already stopped).",
            session_id
        );
    }
    Ok(())
}

#[tauri::command]
pub async fn stop_all_flv_proxy_sessions(
    session_manager: State<'_, FlvProxySessionManager>,
) -> Result<(), String> {
    let sessions = {
        let mut guard = session_manager.0.lock().unwrap();
        guard
            .drain()
            .map(|(_, session)| session)
            .collect::<Vec<_>>()
    };

    for session in sessions {
        println!(
            "[Rust/proxy.rs] stop_all_flv_proxy_sessions: platform={} room={:?} port={} upstream={}",
            session.platform, session.room_id, session.port, session.upstream_url
        );
        session.handle.stop(false).await;
    }
    Ok(())
}

#[tauri::command]
pub async fn start_proxy(
    _app_handle: AppHandle,
    server_handle_state: State<'_, ProxyServerHandle>,
) -> Result<String, String> {
    let port = find_free_port().await;

    // Ensure MutexGuard is dropped before .await
    let existing_handle_to_stop = { server_handle_state.0.lock().unwrap().take() };
    if let Some(existing_handle) = existing_handle_to_stop {
        existing_handle.stop(false).await;
    }

    let server = match HttpServer::new(move || {
        // Create reqwest::Client inside the closure for each worker thread
        let app_data_reqwest_client = web::Data::new(build_proxy_http_client());
        App::new()
            .app_data(app_data_reqwest_client)
            .wrap(actix_cors::Cors::permissive())
            .route("/live.flv", web::get().to(flv_proxy_handler))
            .route("/image", web::get().to(image_proxy_handler))
    })
    .keep_alive(Duration::from_secs(120))
    .bind(("127.0.0.1", port))
    {
        Ok(srv) => srv,
        Err(e) => {
            let err_msg = format!(
                "[Rust/proxy.rs] Failed to bind server to port {}: {}",
                port, e
            );
            eprintln!("{}", err_msg);
            return Err(err_msg);
        }
    }
    .run();

    let server_handle_for_state = server.handle();
    *server_handle_state.0.lock().unwrap() = Some(server_handle_for_state);

    // Use tauri::async_runtime::spawn directly
    tauri::async_runtime::spawn(async move {
        if let Err(e) = server.await {
            eprintln!("[Rust/proxy.rs] Proxy server run error: {}", e);
        } else {
            println!("[Rust/proxy.rs] Proxy server on port {} shut down.", port);
        }
    });

    let base_url = format!("http://127.0.0.1:{}", port);
    Ok(base_url)
}

#[tauri::command]
pub async fn start_static_proxy_server(_app_handle: AppHandle) -> Result<String, String> {
    // Use a dedicated port for static image proxy to avoid interfering with FLV stream proxy
    let port: u16 = 34721;

    // If the server is already running, just return the base URL (idempotent behavior)
    if TcpStream::connect(("127.0.0.1", port)).is_ok() {
        return Ok(format!("http://127.0.0.1:{}", port));
    }

    let server = match HttpServer::new(move || {
        let app_data_reqwest_client = web::Data::new(build_proxy_http_client());
        App::new()
            .app_data(app_data_reqwest_client)
            .wrap(actix_cors::Cors::permissive())
            .route("/live.flv", web::get().to(flv_proxy_handler))
            .route("/image", web::get().to(image_proxy_handler))
    })
    .keep_alive(Duration::from_secs(120))
    .bind(("127.0.0.1", port))
    {
        Ok(srv) => srv,
        Err(e) => {
            // If address already in use, assume server is running and return OK base URL
            if e.kind() == ErrorKind::AddrInUse {
                eprintln!(
                    "[Rust/proxy.rs] Port {} already in use; assuming static proxy running.",
                    port
                );
                return Ok(format!("http://127.0.0.1:{}", port));
            }
            let err_msg = format!(
                "[Rust/proxy.rs] Failed to bind server to port {}: {}",
                port, e
            );
            eprintln!("{}", err_msg);
            return Err(err_msg);
        }
    }
    .run();

    // Do NOT overwrite the main proxy server handle; run static proxy independently

    tauri::async_runtime::spawn(async move {
        if let Err(e) = server.await {
            eprintln!("[Rust/proxy.rs] Proxy server run error: {}", e);
        } else {
            println!("[Rust/proxy.rs] Proxy server on port {} shut down.", port);
        }
    });

    Ok(format!("http://127.0.0.1:{}", port))
}

#[tauri::command]
pub async fn stop_proxy(server_handle_state: State<'_, ProxyServerHandle>) -> Result<(), String> {
    // Ensure MutexGuard is dropped before .await
    let handle_to_stop = { server_handle_state.0.lock().unwrap().take() };

    if let Some(handle) = handle_to_stop {
        handle.stop(false).await; // Changed to non-graceful shutdown
        println!("[Rust/proxy.rs] stop_proxy: Initiated non-graceful shutdown.");
    } else {
        println!("[Rust/proxy.rs] stop_proxy command: No proxy server was running or handle already taken.");
    }
    Ok(())
}
