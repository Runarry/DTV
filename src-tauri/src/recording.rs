use crate::platforms::bilibili::stream_url::get_bilibili_live_stream_url_with_quality;
use crate::platforms::common::types::{GetStreamUrlArgs, GetStreamUrlPayload};
use crate::platforms::common::FollowHttpClient;
use crate::platforms::douyin::douyin_streamer_detail::fetch_douyin_live_stream_info_by_quality;
use crate::platforms::douyu::get_stream_url_with_quality;
use crate::platforms::huya::stream_url::get_huya_unified_with_client;
use chrono::Local;
use futures_util::StreamExt;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex as StdMutex};
use tauri::{async_runtime::JoinHandle, AppHandle, Emitter, State};
use tokio::io::AsyncWriteExt;
use tokio::sync::watch;
use tokio::time::{sleep, Duration, Instant};

const RECORDING_EVENT_NAME: &str = "recording-status";
const DEFAULT_SEGMENT_MINUTES: u32 = 30;
const MIN_SEGMENT_MINUTES: u32 = 1;
const MAX_SEGMENT_MINUTES: u32 = 24 * 60;
const OFFLINE_RETRY_LIMIT: usize = 5;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartLiveRecordingPayload {
    pub platform: String,
    pub room_id: String,
    pub quality: Option<String>,
    pub segment_minutes: Option<u32>,
    pub output_dir: Option<String>,
    pub cookie: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartLiveRecordingResponse {
    pub task_id: String,
    pub resolved_stream_url: String,
    pub output_dir: String,
    pub started_at: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingTaskSnapshot {
    pub task_id: String,
    pub platform: String,
    pub room_id: String,
    pub quality: String,
    pub status: String,
    pub output_dir: String,
    pub current_file: Option<String>,
    pub segment_index: u32,
    pub bytes_written: u64,
    pub started_at: i64,
    pub updated_at: i64,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingStatusEvent {
    pub task_id: String,
    pub platform: String,
    pub room_id: String,
    pub status: String,
    pub current_file: Option<String>,
    pub segment_index: u32,
    pub bytes_written: u64,
    pub message: Option<String>,
    pub timestamp: i64,
}

struct RecordingRuntime {
    platform: String,
    room_id: String,
    state: Arc<StdMutex<RecordingTaskSnapshot>>,
    stop_tx: watch::Sender<bool>,
    join_handle: JoinHandle<()>,
}

#[derive(Default)]
pub struct RecordingManager(pub StdMutex<HashMap<String, RecordingRuntime>>);

#[tauri::command]
pub async fn get_recording_output_dir_default() -> Result<String, String> {
    Ok(default_output_dir().to_string_lossy().to_string())
}

#[tauri::command]
pub async fn list_live_recordings(
    manager: State<'_, RecordingManager>,
) -> Result<Vec<RecordingTaskSnapshot>, String> {
    let snapshots = {
        let guard = manager
            .0
            .lock()
            .map_err(|_| "recording manager lock poisoned".to_string())?;
        guard
            .values()
            .filter_map(|runtime| snapshot_clone(&runtime.state))
            .collect::<Vec<_>>()
    };
    Ok(snapshots)
}

#[tauri::command]
pub async fn start_live_recording(
    app_handle: AppHandle,
    manager: State<'_, RecordingManager>,
    follow_http: State<'_, FollowHttpClient>,
    payload: StartLiveRecordingPayload,
) -> Result<StartLiveRecordingResponse, String> {
    let platform = normalize_platform(&payload.platform)
        .ok_or_else(|| format!("Unsupported platform: {}", payload.platform))?;
    let room_id = payload.room_id.trim().to_string();
    if room_id.is_empty() {
        return Err("room_id is required".to_string());
    }
    let quality = normalize_quality(payload.quality.as_deref());
    let segment_minutes = normalize_segment_minutes(payload.segment_minutes);
    let cookie = payload.cookie.clone().filter(|v| !v.trim().is_empty());

    let output_root = resolve_output_root(payload.output_dir.as_deref())?;
    let output_dir = build_platform_room_output_dir(&output_root, platform, &room_id);
    tokio::fs::create_dir_all(&output_dir)
        .await
        .map_err(|e| format!("Failed to create output dir: {}", e))?;

    let existing_task_id = {
        let guard = manager
            .0
            .lock()
            .map_err(|_| "recording manager lock poisoned".to_string())?;
        guard.iter().find_map(|(task_id, runtime)| {
            if runtime.platform == platform
                && runtime.room_id == room_id
                && snapshot_clone(&runtime.state)
                    .map(|snap| is_active_status(&snap.status))
                    .unwrap_or(false)
            {
                Some(task_id.clone())
            } else {
                None
            }
        })
    };
    if let Some(task_id) = existing_task_id {
        return Err(format!(
            "Recording already running for {}:{} (task_id={})",
            platform, room_id, task_id
        ));
    }

    let follow_client = follow_http.0.inner.clone();
    let initial_url = resolve_stream_url(
        platform,
        &room_id,
        &quality,
        cookie.as_deref(),
        &follow_client,
    )
    .await?;

    let task_id = generate_task_id();
    let started_at = now_millis();
    let snapshot = RecordingTaskSnapshot {
        task_id: task_id.clone(),
        platform: platform.to_string(),
        room_id: room_id.clone(),
        quality: quality.clone(),
        status: "starting".to_string(),
        output_dir: output_dir.to_string_lossy().to_string(),
        current_file: None,
        segment_index: 0,
        bytes_written: 0,
        started_at,
        updated_at: started_at,
        message: None,
    };
    let state = Arc::new(StdMutex::new(snapshot));
    emit_status_event(&app_handle, &state);

    let (stop_tx, stop_rx) = watch::channel(false);
    let worker_app_handle = app_handle.clone();
    let worker_platform = platform.to_string();
    let worker_room_id = room_id.clone();
    let worker_quality = quality.clone();
    let worker_cookie = cookie.clone();
    let worker_output_dir = output_dir.clone();
    let worker_state = state.clone();
    let worker_follow_client = follow_client.clone();
    let worker_initial_url = initial_url.clone();
    let worker_join = tauri::async_runtime::spawn(async move {
        run_recording_worker(
            worker_app_handle,
            worker_platform,
            worker_room_id,
            worker_quality,
            worker_cookie,
            worker_output_dir,
            segment_minutes,
            worker_state,
            stop_rx,
            worker_follow_client,
            worker_initial_url,
        )
        .await;
    });

    {
        let mut guard = manager
            .0
            .lock()
            .map_err(|_| "recording manager lock poisoned".to_string())?;
        guard.insert(
            task_id.clone(),
            RecordingRuntime {
                platform: platform.to_string(),
                room_id,
                state,
                stop_tx,
                join_handle: worker_join,
            },
        );
    }

    Ok(StartLiveRecordingResponse {
        task_id,
        resolved_stream_url: initial_url,
        output_dir: output_dir.to_string_lossy().to_string(),
        started_at,
    })
}

#[tauri::command]
pub async fn stop_live_recording(
    manager: State<'_, RecordingManager>,
    task_id: String,
) -> Result<(), String> {
    let runtime = {
        let mut guard = manager
            .0
            .lock()
            .map_err(|_| "recording manager lock poisoned".to_string())?;
        guard.remove(&task_id)
    };

    if let Some(runtime) = runtime {
        let _ = runtime.stop_tx.send(true);
        let _ = runtime.join_handle.await;
    }

    Ok(())
}

#[tauri::command]
pub async fn stop_all_live_recordings(manager: State<'_, RecordingManager>) -> Result<(), String> {
    let runtimes = {
        let mut guard = manager
            .0
            .lock()
            .map_err(|_| "recording manager lock poisoned".to_string())?;
        guard
            .drain()
            .map(|(_, runtime)| runtime)
            .collect::<Vec<_>>()
    };

    for runtime in runtimes {
        let _ = runtime.stop_tx.send(true);
        let _ = runtime.join_handle.await;
    }

    Ok(())
}

async fn run_recording_worker(
    app_handle: AppHandle,
    platform: String,
    room_id: String,
    quality: String,
    cookie: Option<String>,
    output_dir: PathBuf,
    segment_minutes: u32,
    state: Arc<StdMutex<RecordingTaskSnapshot>>,
    mut stop_rx: watch::Receiver<bool>,
    follow_client: reqwest::Client,
    mut current_stream_url: String,
) {
    let mut segment_index: u32 = 0;
    let segment_duration = Duration::from_secs(u64::from(segment_minutes) * 60);
    let mut reconnect_attempts: usize = 0;
    let mut offline_attempts: usize = 0;
    let mut should_stop = false;

    while !should_stop {
        if *stop_rx.borrow() {
            mark_status(&state, "stopped", Some("stopped_by_user".to_string()));
            emit_status_event(&app_handle, &state);
            break;
        }

        let request_builder = build_stream_request(
            &follow_client,
            &platform,
            &current_stream_url,
            cookie.as_deref(),
        );
        let response = match request_builder.send().await {
            Ok(resp) => resp,
            Err(e) => {
                reconnect_attempts += 1;
                mark_status(
                    &state,
                    "reconnecting",
                    Some(format!("connect_failed: {}", e)),
                );
                emit_status_event(&app_handle, &state);
                sleep(backoff_duration(reconnect_attempts)).await;
                match resolve_stream_url(
                    &platform,
                    &room_id,
                    &quality,
                    cookie.as_deref(),
                    &follow_client,
                )
                .await
                {
                    Ok(url) => {
                        current_stream_url = url;
                        continue;
                    }
                    Err(err) => {
                        if is_room_offline_error(&err) {
                            offline_attempts += 1;
                            if offline_attempts >= OFFLINE_RETRY_LIMIT {
                                mark_status(
                                    &state,
                                    "stopped",
                                    Some("stream_ended_or_offline".to_string()),
                                );
                                emit_status_event(&app_handle, &state);
                                break;
                            }
                        }
                        mark_status(&state, "reconnecting", Some(err));
                        emit_status_event(&app_handle, &state);
                        continue;
                    }
                }
            }
        };

        if !response.status().is_success() {
            reconnect_attempts += 1;
            let code = response.status();
            let message = format!("upstream_status={}", code);
            mark_status(&state, "reconnecting", Some(message));
            emit_status_event(&app_handle, &state);
            sleep(backoff_duration(reconnect_attempts)).await;
            continue;
        }

        reconnect_attempts = 0;
        offline_attempts = 0;
        mark_status(&state, "recording", None);
        emit_status_event(&app_handle, &state);

        segment_index += 1;
        let mut file =
            match open_segment_file(&output_dir, &platform, &room_id, segment_index).await {
                Ok(file) => file,
                Err(e) => {
                    mark_status(
                        &state,
                        "failed",
                        Some(format!("failed_to_open_output_file: {}", e)),
                    );
                    emit_status_event(&app_handle, &state);
                    break;
                }
            };
        let mut segment_started = Instant::now();
        set_current_file(&state, file.1.clone(), segment_index);
        emit_status_event(&app_handle, &state);

        let mut stream = response.bytes_stream();
        while let Some(chunk_result) = stream.next().await {
            if *stop_rx.borrow() {
                should_stop = true;
                break;
            }

            let chunk = match chunk_result {
                Ok(c) => c,
                Err(e) => {
                    mark_status(
                        &state,
                        "reconnecting",
                        Some(format!("stream_read_error: {}", e)),
                    );
                    emit_status_event(&app_handle, &state);
                    break;
                }
            };

            if segment_started.elapsed() >= segment_duration {
                if let Err(e) = file.0.flush().await {
                    mark_status(&state, "failed", Some(format!("flush_failed: {}", e)));
                    emit_status_event(&app_handle, &state);
                    should_stop = true;
                    break;
                }

                segment_index += 1;
                match open_segment_file(&output_dir, &platform, &room_id, segment_index).await {
                    Ok(next_file) => {
                        file = next_file;
                        segment_started = Instant::now();
                        set_current_file(&state, file.1.clone(), segment_index);
                        emit_status_event(&app_handle, &state);
                    }
                    Err(e) => {
                        mark_status(
                            &state,
                            "failed",
                            Some(format!("failed_to_open_next_segment: {}", e)),
                        );
                        emit_status_event(&app_handle, &state);
                        should_stop = true;
                        break;
                    }
                }
            }

            if let Err(e) = file.0.write_all(&chunk).await {
                mark_status(&state, "failed", Some(format!("write_failed: {}", e)));
                emit_status_event(&app_handle, &state);
                should_stop = true;
                break;
            }
            increment_bytes_written(&state, chunk.len() as u64);
        }

        if should_stop {
            break;
        }

        if let Err(e) = file.0.flush().await {
            mark_status(&state, "failed", Some(format!("flush_failed: {}", e)));
            emit_status_event(&app_handle, &state);
            break;
        }

        match resolve_stream_url(
            &platform,
            &room_id,
            &quality,
            cookie.as_deref(),
            &follow_client,
        )
        .await
        {
            Ok(url) => {
                current_stream_url = url;
                mark_status(&state, "reconnecting", Some("stream_reconnect".to_string()));
                emit_status_event(&app_handle, &state);
                sleep(Duration::from_secs(2)).await;
            }
            Err(err) => {
                if is_room_offline_error(&err) {
                    offline_attempts += 1;
                    if offline_attempts >= OFFLINE_RETRY_LIMIT {
                        mark_status(
                            &state,
                            "stopped",
                            Some("stream_ended_or_offline".to_string()),
                        );
                        emit_status_event(&app_handle, &state);
                        break;
                    }
                }
                mark_status(&state, "reconnecting", Some(err));
                emit_status_event(&app_handle, &state);
                sleep(Duration::from_secs(3)).await;
            }
        }
    }

    if *stop_rx.borrow() {
        mark_status(&state, "stopped", Some("stopped_by_user".to_string()));
        emit_status_event(&app_handle, &state);
    } else if let Some(snapshot) = snapshot_clone(&state) {
        if snapshot.status != "failed" && snapshot.status != "stopped" {
            mark_status(&state, "stopped", Some("worker_exit".to_string()));
            emit_status_event(&app_handle, &state);
        }
    }
}

fn resolve_output_root(custom_output_dir: Option<&str>) -> Result<PathBuf, String> {
    if let Some(raw) = custom_output_dir {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return Ok(PathBuf::from(trimmed));
        }
    }
    Ok(default_output_dir())
}

fn default_output_dir() -> PathBuf {
    if let Some(video_dir) = dirs::video_dir() {
        return video_dir.join("DTV");
    }
    if let Ok(current) = std::env::current_dir() {
        return current.join("recordings");
    }
    PathBuf::from("./recordings")
}

fn build_platform_room_output_dir(root: &Path, platform: &str, room_id: &str) -> PathBuf {
    root.join(platform).join(sanitize_token(room_id))
}

fn generate_task_id() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

fn normalize_platform(input: &str) -> Option<&'static str> {
    match input.trim().to_ascii_uppercase().as_str() {
        "DOUYU" => Some("DOUYU"),
        "DOUYIN" => Some("DOUYIN"),
        "HUYA" => Some("HUYA"),
        "BILIBILI" => Some("BILIBILI"),
        _ => None,
    }
}

fn normalize_quality(input: Option<&str>) -> String {
    match input.unwrap_or("原画").trim() {
        "高清" => "高清".to_string(),
        "标清" => "标清".to_string(),
        _ => "原画".to_string(),
    }
}

fn normalize_segment_minutes(input: Option<u32>) -> u32 {
    match input {
        Some(value) if value < MIN_SEGMENT_MINUTES => DEFAULT_SEGMENT_MINUTES,
        Some(value) if value > MAX_SEGMENT_MINUTES => MAX_SEGMENT_MINUTES,
        Some(value) => value,
        None => DEFAULT_SEGMENT_MINUTES,
    }
}

fn sanitize_token(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    let trimmed = out.trim_matches('_');
    if trimmed.is_empty() {
        "unknown".to_string()
    } else {
        trimmed.to_string()
    }
}

fn now_millis() -> i64 {
    Local::now().timestamp_millis()
}

fn is_active_status(status: &str) -> bool {
    matches!(status, "starting" | "recording" | "reconnecting")
}

fn is_room_offline_error(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    message.contains("未开播")
        || message.contains("房间不存在")
        || message.contains("stream_ended_or_offline")
        || lower.contains("offline")
}

fn build_stream_request(
    client: &reqwest::Client,
    platform: &str,
    url: &str,
    cookie: Option<&str>,
) -> reqwest::RequestBuilder {
    let mut request = client
        .get(url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36",
        )
        .header("Accept", "video/x-flv,application/octet-stream,*/*")
        .header("Connection", "keep-alive")
        .header("Range", "bytes=0-");

    request = match platform {
        "HUYA" => request
            .header("Referer", "https://www.huya.com/")
            .header("Origin", "https://www.huya.com"),
        "BILIBILI" => request
            .header("Referer", "https://live.bilibili.com/")
            .header("Origin", "https://live.bilibili.com"),
        "DOUYIN" => request
            .header("Referer", "https://live.douyin.com/")
            .header("Origin", "https://live.douyin.com"),
        "DOUYU" => request.header("Referer", "https://www.douyu.com/"),
        _ => request,
    };

    if let Some(raw_cookie) = cookie {
        let trimmed = raw_cookie.trim();
        if !trimmed.is_empty() {
            request = request.header("Cookie", trimmed);
        }
    }

    request
}

fn backoff_duration(attempt: usize) -> Duration {
    match attempt {
        0 | 1 => Duration::from_secs(2),
        2 => Duration::from_secs(5),
        3 => Duration::from_secs(10),
        _ => Duration::from_secs(30),
    }
}

async fn resolve_stream_url(
    platform: &str,
    room_id: &str,
    quality: &str,
    cookie: Option<&str>,
    follow_client: &reqwest::Client,
) -> Result<String, String> {
    let raw_url = match platform {
        "DOUYU" => get_stream_url_with_quality(room_id, quality, None)
            .await
            .map_err(|e| format!("Douyu stream url failed: {}", e))?,
        "DOUYIN" => {
            let info = fetch_douyin_live_stream_info_by_quality(room_id, quality).await?;
            if info.status != Some(2) || info.stream_url.is_none() {
                return Err("Douyin streamer is offline or stream unavailable".to_string());
            }
            info.stream_url.unwrap_or_default()
        }
        "HUYA" => {
            let response =
                get_huya_unified_with_client(follow_client, room_id, Some(quality), None).await?;
            if !response.is_live {
                return Err("Huya streamer is offline".to_string());
            }
            response
                .selected_url
                .or_else(|| response.flv_tx_urls.first().map(|item| item.url.clone()))
                .ok_or_else(|| "Huya stream url unavailable".to_string())?
        }
        "BILIBILI" => {
            let info = get_bilibili_live_stream_url_with_quality(
                GetStreamUrlPayload {
                    args: GetStreamUrlArgs {
                        room_id_str: room_id.to_string(),
                    },
                },
                quality.to_string(),
                cookie.map(|v| v.to_string()),
            )
            .await?;

            if info.status != Some(1) {
                return Err("Bilibili streamer is offline".to_string());
            }
            let url = info
                .stream_url
                .ok_or_else(|| "Bilibili stream url unavailable".to_string())?;
            if !is_flv_stream(&url) {
                return Err(
                    "Bilibili current stream is HLS; recording supports FLV only".to_string(),
                );
            }
            url
        }
        _ => return Err(format!("Unsupported platform: {}", platform)),
    };

    Ok(force_https(&raw_url))
}

fn is_flv_stream(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    lower.contains(".flv") || lower.contains("flv?")
}

fn force_https(url: &str) -> String {
    if url.starts_with("https://") {
        return url.to_string();
    }
    if let Some(rest) = url.strip_prefix("http://") {
        return format!("https://{}", rest);
    }
    url.to_string()
}

async fn open_segment_file(
    output_dir: &Path,
    platform: &str,
    room_id: &str,
    segment_index: u32,
) -> Result<(tokio::fs::File, String), String> {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let filename = format!(
        "{}_{}_{}_part{:03}.flv",
        platform.to_ascii_lowercase(),
        sanitize_token(room_id),
        timestamp,
        segment_index
    );
    let path = output_dir.join(filename);
    let file = tokio::fs::File::create(&path)
        .await
        .map_err(|e| format!("failed to create segment file: {}", e))?;
    Ok((file, path.to_string_lossy().to_string()))
}

fn mark_status(
    state: &Arc<StdMutex<RecordingTaskSnapshot>>,
    status: &str,
    message: Option<String>,
) {
    if let Ok(mut snapshot) = state.lock() {
        snapshot.status = status.to_string();
        snapshot.updated_at = now_millis();
        snapshot.message = message;
    }
}

fn set_current_file(
    state: &Arc<StdMutex<RecordingTaskSnapshot>>,
    current_file: String,
    segment_index: u32,
) {
    if let Ok(mut snapshot) = state.lock() {
        snapshot.current_file = Some(current_file);
        snapshot.segment_index = segment_index;
        snapshot.updated_at = now_millis();
    }
}

fn increment_bytes_written(state: &Arc<StdMutex<RecordingTaskSnapshot>>, size: u64) {
    if let Ok(mut snapshot) = state.lock() {
        snapshot.bytes_written = snapshot.bytes_written.saturating_add(size);
        snapshot.updated_at = now_millis();
    }
}

fn snapshot_clone(state: &Arc<StdMutex<RecordingTaskSnapshot>>) -> Option<RecordingTaskSnapshot> {
    state.lock().ok().map(|snapshot| snapshot.clone())
}

fn emit_status_event(app_handle: &AppHandle, state: &Arc<StdMutex<RecordingTaskSnapshot>>) {
    if let Some(snapshot) = snapshot_clone(state) {
        let payload = RecordingStatusEvent {
            task_id: snapshot.task_id.clone(),
            platform: snapshot.platform.clone(),
            room_id: snapshot.room_id.clone(),
            status: snapshot.status.clone(),
            current_file: snapshot.current_file.clone(),
            segment_index: snapshot.segment_index,
            bytes_written: snapshot.bytes_written,
            message: snapshot.message.clone(),
            timestamp: now_millis(),
        };
        let _ = app_handle.emit(RECORDING_EVENT_NAME, payload);
    }
}
