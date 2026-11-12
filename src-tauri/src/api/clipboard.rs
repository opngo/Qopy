use base64::{ engine::general_purpose::STANDARD, Engine };
use lazy_static::lazy_static;
use rdev::{ simulate, EventType, Key };
use regex::Regex;
use sha2::{ Digest, Sha256 };
use sqlx::SqlitePool;
use std::fs;
use std::sync::atomic::{ AtomicBool, AtomicU64, Ordering };
use std::{ thread, time::Duration };
use tauri::{ AppHandle, Emitter, Listener, Manager };
use tauri_plugin_aptabase::EventTracker;
use tauri_plugin_clipboard::Clipboard;
use tokio::runtime::Runtime as TokioRuntime;
use url::Url;
use uuid::Uuid;

use crate::db;
use crate::utils::commands::get_app_info;
use crate::utils::favicon::fetch_favicon_as_base64;
use crate::utils::types::{ ContentType, HistoryItem };

lazy_static! {
    /// True while we are programmatically writing the clipboard and
    /// simulating a paste. The monitor must ignore those changes, otherwise
    /// our own paste would be recorded as a fresh clipboard entry.
    static ref IS_PROGRAMMATIC_PASTE: AtomicBool = AtomicBool::new(false);

    /// Fingerprint of the last clipboard payload we actually processed.
    /// Clipboard monitors can fire multiple times for a single change (and
    /// some platforms fire them even when nothing changed); comparing the
    /// fingerprint lets us drop duplicates instead of re-recording the same
    /// content over and over.
    static ref LAST_CLIPBOARD: parking_lot::Mutex<u64> = parking_lot::Mutex::new(0);

    /// When the current programmatic paste started (ms since epoch). Used to
    /// detect a stuck IS_PROGRAMMATIC_PASTE flag - a stuck flag would make
    /// the monitor ignore every future copy forever.
    static ref LAST_PASTE_START: AtomicU64 = AtomicU64::new(0);
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn is_pasting() -> bool {
    IS_PROGRAMMATIC_PASTE.load(Ordering::SeqCst)
}

fn content_fingerprint(type_label: &str, content: &str) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(type_label.as_bytes());
    hasher.update([0u8]);
    hasher.update(content.as_bytes());
    let digest = hasher.finalize();
    u64::from_be_bytes(digest[..8].try_into().expect("sha256 digest is 32 bytes"))
}

/// Shared clipboard writer for both "write and paste" and "copy only".
///
/// NOTE: the content type comes from the frontend enum, where files are
/// serialized as "file" (singular) and code exists as its own type. The old
/// implementation only handled "files" and had no "code" arm, so pasting a
/// code or file entry silently failed with "Unsupported content type".
fn write_to_clipboard(
    app_handle: &AppHandle,
    content: &str,
    content_type: &str
) -> Result<(), String> {
    let clipboard = app_handle.state::<Clipboard>();

    match content_type {
        "text" | "link" | "color" | "code" => clipboard
            .write_text(content.to_string())
            .map_err(|e| e.to_string()),
        "image" => clipboard
            .write_image_base64(content.to_string())
            .map_err(|e| e.to_string()),
        "file" | "files" => clipboard
            .write_files_uris(
                content
                    .split(", ")
                    .map(|file| file.to_string())
                    .collect::<Vec<String>>()
            )
            .map_err(|e| e.to_string()),
        _ => Err("Unsupported content type".to_string()),
    }
}

#[tauri::command]
pub async fn write_and_paste(
    app_handle: AppHandle,
    content: String,
    content_type: String
) -> Result<(), String> {
    write_to_clipboard(&app_handle, &content, &content_type)?;

    LAST_PASTE_START.store(now_ms(), Ordering::SeqCst);
    IS_PROGRAMMATIC_PASTE.store(true, Ordering::SeqCst);

    thread::spawn(|| {
        // Give the clipboard manager time to fully propagate the write
        // before simulating the paste keystroke.
        thread::sleep(Duration::from_millis(250));

        #[cfg(target_os = "macos")]
        let modifier_key = Key::MetaLeft;
        #[cfg(not(target_os = "macos"))]
        let modifier_key = Key::ControlLeft;

        let events = vec![
            EventType::KeyPress(modifier_key),
            EventType::KeyPress(Key::KeyV),
            EventType::KeyRelease(Key::KeyV),
            EventType::KeyRelease(modifier_key)
        ];

        for event in events {
            if let Err(e) = simulate(&event) {
                println!("Simulation error: {:?}", e);
            }
            thread::sleep(Duration::from_millis(20));
        }
    });

    tokio::spawn(async move {
        // Hold the suppression flag long enough to cover the simulated
        // keystrokes and any echo from the clipboard manager.
        tokio::time::sleep(tokio::time::Duration::from_millis(1200)).await;
        IS_PROGRAMMATIC_PASTE.store(false, Ordering::SeqCst);

        // Invalidate the fingerprint so a genuine re-copy of the same
        // content right after a paste is still recorded.
        *LAST_CLIPBOARD.lock() = 0;
    });

    let _ = app_handle.track_event(
        "clipboard_paste",
        Some(serde_json::json!({
        "content_type": content_type
    }))
    );

    Ok(())
}

/// Writes the content to the clipboard WITHOUT simulating a paste.
/// Used by the "Copy to clipboard" action.
#[tauri::command]
pub async fn write_clipboard(
    app_handle: AppHandle,
    content: String,
    content_type: String
) -> Result<(), String> {
    write_to_clipboard(&app_handle, &content, &content_type)?;

    // Invalidate the fingerprint so a genuine re-copy of this same content
    // from another app right afterwards is still recorded.
    *LAST_CLIPBOARD.lock() = 0;

    let _ = app_handle.track_event(
        "clipboard_copy_only",
        Some(serde_json::json!({ "content_type": content_type }))
    );

    Ok(())
}

enum Payload {
    Image(String),
    Files(Vec<String>),
    Text(String),
}

pub fn setup(app: &AppHandle) {
    let app_handle = app.clone();
    let runtime = TokioRuntime::new().expect("Failed to create Tokio runtime");

    app_handle.clone().listen("plugin:clipboard://clipboard-monitor/update", move |_event| {
        let app_handle = app_handle.clone();
        runtime.block_on(async move {
            // Our own programmatic write (paste or copy-only) - ignore it.
            // Safety: if the flag ever gets stuck (e.g. the reset task was
            // delayed), never let it block real copies for more than 3s.
            if IS_PROGRAMMATIC_PASTE.load(Ordering::SeqCst) {
                let started = LAST_PASTE_START.load(Ordering::SeqCst);
                if started != 0 && now_ms().saturating_sub(started) < 3000 {
                    return;
                }
                IS_PROGRAMMATIC_PASTE.store(false, Ordering::SeqCst);
                *LAST_CLIPBOARD.lock() = 0;
            }

            let clipboard = app_handle.state::<Clipboard>();
            let available_types = match clipboard.available_types() {
                Ok(t) => t,
                Err(e) => {
                    println!("Failed to read clipboard types: {}", e);
                    return;
                }
            };

            let payload = if available_types.image {
                match clipboard.read_image_base64() {
                    Ok(b64) => Some(Payload::Image(b64)),
                    Err(e) => {
                        println!("Failed to read image: {}", e);
                        None
                    }
                }
            } else if available_types.files {
                match clipboard.read_files() {
                    Ok(files) => Some(Payload::Files(files)),
                    Err(e) => {
                        println!("Failed to read files: {}", e);
                        None
                    }
                }
            } else if available_types.text {
                match clipboard.read_text() {
                    Ok(text) => Some(Payload::Text(text.to_string())),
                    Err(e) => {
                        println!("Failed to read text: {}", e);
                        None
                    }
                }
            } else {
                None
            };

            let payload = match payload {
                Some(p) => p,
                None => return,
            };

            let files_joined;
            let (type_label, fingerprint_content) = match &payload {
                Payload::Image(b64) => ("image", b64.as_str()),
                Payload::Files(files) => {
                    files_joined = files.join(", ");
                    ("files", files_joined.as_str())
                }
                Payload::Text(text) => ("text", text.as_str()),
            };

            let fingerprint = content_fingerprint(type_label, fingerprint_content);
            if *LAST_CLIPBOARD.lock() == fingerprint {
                // Duplicate / spurious monitor event for the same content:
                // skip it so the UI selection is never disturbed.
                return;
            }
            *LAST_CLIPBOARD.lock() = fingerprint;

            let (app_name, app_icon) = get_app_info();

            match get_pool(&app_handle).await {
                Ok(pool) => match payload {
                    Payload::Image(image_data) => {
                        println!("Handling image change");
                        if let Ok(file_path) = save_image_to_file(&app_handle, &image_data).await {
                            let _ = db::history::add_history_item(
                                app_handle.clone(),
                                pool,
                                HistoryItem::new(
                                    app_name,
                                    ContentType::Image,
                                    file_path,
                                    None,
                                    app_icon,
                                    None
                                )
                            ).await;
                        } else {
                            println!("Failed to save clipboard image");
                        }
                    }
                    Payload::Files(files) => {
                        println!("Handling files change");
                        for file in files {
                            let _ = db::history::add_history_item(
                                app_handle.clone(),
                                pool.clone(),
                                HistoryItem::new(
                                    app_name.clone(),
                                    ContentType::File,
                                    file,
                                    None,
                                    app_icon.clone(),
                                    None
                                )
                            ).await;
                        }
                    }
                    Payload::Text(text) => {
                        if text.is_empty() {
                            return;
                        }

                        println!("Handling text change");
                        let url_regex = Regex::new(
                            r"^https?://(?:www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b(?:[-a-zA-Z0-9()@:%_+\+.~#?&//=]*)$"
                        ).unwrap();

                        if url_regex.is_match(&text) {
                            if let Ok(url) = Url::parse(&text) {
                                // Time-box the favicon fetch: a slow network
                                // must never hold up recording the link.
                                let favicon = match tokio::time::timeout(
                                    std::time::Duration::from_secs(4),
                                    fetch_favicon_as_base64(url),
                                )
                                .await
                                {
                                    Ok(Ok(Some(f))) => Some(f),
                                    _ => None,
                                };

                                let _ = db::history::add_history_item(
                                    app_handle.clone(),
                                    pool,
                                    HistoryItem::new(
                                        app_name,
                                        ContentType::Link,
                                        text,
                                        favicon,
                                        app_icon,
                                        None
                                    )
                                ).await;
                            }
                        } else if crate::utils::commands::detect_color(&text) {
                            let _ = db::history::add_history_item(
                                app_handle.clone(),
                                pool,
                                HistoryItem::new(
                                    app_name,
                                    ContentType::Color,
                                    text,
                                    None,
                                    app_icon,
                                    None
                                )
                            ).await;
                        } else if let Some(language) = crate::utils::code::detect_code(&text) {
                            let _ = db::history::add_history_item(
                                app_handle.clone(),
                                pool,
                                HistoryItem::new(
                                    app_name,
                                    ContentType::Code,
                                    text,
                                    None,
                                    app_icon,
                                    Some(language)
                                )
                            ).await;
                        } else {
                            let _ = db::history::add_history_item(
                                app_handle.clone(),
                                pool,
                                HistoryItem::new(
                                    app_name,
                                    ContentType::Text,
                                    text.clone(),
                                    None,
                                    app_icon,
                                    None
                                )
                            ).await;
                        }
                    }
                },
                Err(e) => {
                    println!("Failed to get database pool: {}", e);
                }
            }

            let _ = app_handle.emit("clipboard-content-updated", ());
            let _ = app_handle.track_event(
                "clipboard_copied",
                Some(
                    serde_json::json!({
                    "content_type": if available_types.image { "image" }
                        else if available_types.files { "files" }
                        else if available_types.text { "text" }
                        else { "unknown" }
                })
                )
            );
        });
    });
}

async fn get_pool(
    app_handle: &AppHandle
) -> Result<tauri::State<'_, SqlitePool>, Box<dyn std::error::Error + Send + Sync>> {
    Ok(app_handle.state::<SqlitePool>())
}

#[tauri::command]
pub fn start_monitor(app_handle: AppHandle) -> Result<(), String> {
    let clipboard = app_handle.state::<Clipboard>();
    clipboard.start_monitor(app_handle.clone()).map_err(|e| e.to_string())?;
    app_handle
        .emit("plugin:clipboard://clipboard-monitor/status", true)
        .map_err(|e| e.to_string())?;
    Ok(())
}

async fn save_image_to_file(
    app_handle: &AppHandle,
    base64_data: &str
) -> Result<String, Box<dyn std::error::Error>> {
    let app_data_dir = app_handle.path().app_data_dir().unwrap();
    let images_dir = app_data_dir.join("images");
    fs::create_dir_all(&images_dir)?;

    let file_name = format!("{}.png", Uuid::new_v4());
    let file_path = images_dir.join(&file_name);

    let bytes = STANDARD.decode(base64_data)?;
    fs::write(&file_path, bytes)?;

    Ok(file_path.to_string_lossy().into_owned())
}
