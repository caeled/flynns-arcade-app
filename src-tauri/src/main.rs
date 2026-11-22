// Prevents an extra console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::webview::PageLoadEvent;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_opener::OpenerExt;

/// Returns true when `url` belongs to the app itself and may load inside
/// the window: the Tauri production scheme (`tauri://localhost/...` on
/// macOS), the Windows production origin (`http://tauri.localhost/...`),
/// and inline schemes. Everything else is not the app's own content.
/// (Same test the main Everyday Cool app uses.)
fn is_app_url(url: &tauri::Url) -> bool {
    match url.scheme() {
        "tauri" => true,
        "http" | "https" => url.host_str() == Some("tauri.localhost"),
        "about" | "data" | "blob" => true,
        _ => false,
    }
}

/// ------------------------------------------------------------------
/// Clear Cache (File menu). The same cure as the main Everyday Cool
/// app's v1.1.6 item: a persisted webview profile can keep serving
/// pre-update pages — a room's service worker answers cache-first out
/// of Cache Storage, and neither reinstalling nor restarting clears
/// that profile.
///
/// The item cannot clear the cache in place: the running webview owns
/// those files. So the click writes a flag file and restarts the app,
/// and at the very top of setup — before any window (and therefore any
/// webview) exists — the flag is consumed: only the cache-bearing
/// subdirectories of the profile are deleted (HTTP cache, code cache,
/// GPU cache, and the Service Worker directory, which holds the
/// service worker registrations and their Cache Storage). Local
/// Storage and IndexedDB are never touched, so game progress and
/// preferences survive a Clear Cache. The Arcade keeps no per-user
/// data, so there is exactly one profile to clear.
static CACHE_JUST_CLEARED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

const CLEAR_CACHE_FLAG: &str = "clear-cache.flag";

/// Path of a small flag file in the app local data directory.
fn flag_path(handle: &AppHandle, name: &str) -> Option<std::path::PathBuf> {
    handle
        .path()
        .app_local_data_dir()
        .ok()
        .map(|dir| dir.join(name))
}

fn write_flag(handle: &AppHandle, name: &str) {
    if let Some(flag) = flag_path(handle, name) {
        if let Some(parent) = flag.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&flag, b"1");
    }
}

/// File → Clear Cache: leave the flag for the next launch, then
/// restart. The deletion happens in setup of the new process, before
/// its webview is created (see consume_clear_cache_flag).
fn request_clear_cache(handle: &AppHandle) {
    write_flag(handle, CLEAR_CACHE_FLAG);
    handle.restart();
}

/// Consume a Clear Cache flag left by the previous run. Returns true
/// when a flag was present (the caller then arms the "cache cleared"
/// notice for the main window). Deletion is best-effort with a short
/// retry: the previous process may still be releasing file locks in
/// the first moments after a restart.
fn consume_clear_cache_flag(handle: &AppHandle) -> bool {
    let Some(flag) = flag_path(handle, CLEAR_CACHE_FLAG) else {
        return false;
    };
    if !flag.exists() {
        return false;
    }
    let _ = std::fs::remove_file(&flag);

    let mut targets: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(data_dir) = handle.path().app_local_data_dir() {
        // The WebView2 profile's Default folder lives under
        // <app local data>/EBWebView. Only cache directories are
        // listed — Local Storage and IndexedDB live in sibling
        // directories and are preserved.
        let profile = data_dir.join("EBWebView").join("Default");
        for sub in ["Service Worker", "Cache", "Code Cache", "GPUCache"] {
            targets.push(profile.join(sub));
        }
    }
    #[cfg(target_os = "macos")]
    {
        // WKWebView has no EBWebView; the app's system cache container
        // (~/Library/Caches/<bundle id>) is pure cache and regenerates.
        if let Ok(home) = handle.path().home_dir() {
            targets.push(
                home.join("Library")
                    .join("Caches")
                    .join(handle.config().identifier.as_str()),
            );
        }
    }
    for target in targets {
        for _ in 0..5 {
            if !target.exists() || std::fs::remove_dir_all(&target).is_ok() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(150));
        }
    }
    true
}

/// In-page notice shown once on the main window after a Clear Cache
/// restart, so the click never feels dead. Injected from Rust; the
/// bundled pages are untouched.
const CACHE_CLEARED_NOTICE_JS: &str = "(() => { const d = document.createElement('div'); d.textContent = 'Cache cleared \u{2014} the arcade now loads fresh copies of every game. Your saved progress was not touched.'; d.style.cssText = 'position:fixed;top:14px;left:50%;transform:translateX(-50%);z-index:99999;background:#1d3b2a;color:#eaf5ec;padding:10px 18px;border-radius:10px;font:14px system-ui,sans-serif;box-shadow:0 4px 18px rgba(0,0,0,.35)'; document.body.appendChild(d); setTimeout(() => d.remove(), 8000); })();";

/// Navigate the main window to a bundled page at the app root
/// (documents.html, about.html): resolve the path against the window's
/// current URL so the jump works from any game. It is an app URL, so
/// the navigation handler keeps it inside the window.
fn navigate_main(app: &AppHandle, path: &str) {
    if let Some(win) = app.get_webview_window("main") {
        if let Ok(current) = win.url() {
            if let Ok(target) = current.join(path) {
                let _ = win.navigate(target);
            }
        }
        let _ = win.set_focus();
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .menu(|handle| {
            // The main app's File | Help shape, adapted: the Arcade
            // keeps no per-user data, so the File menu carries only
            // Clear Cache and Exit; Help carries the Game Documents
            // page (every game's lesson, generated from the manifest
            // at sync time), the repository, and About.
            let clear_cache =
                MenuItem::with_id(handle, "clear-cache", "Clear Cache", true, None::<&str>)?;
            let game_documents = MenuItem::with_id(
                handle,
                "game-documents",
                "Cabinet Documents",
                true,
                None::<&str>,
            )?;
            let github = MenuItem::with_id(
                handle,
                "github",
                "GitHub Repository",
                true,
                None::<&str>,
            )?;
            let about = MenuItem::with_id(
                handle,
                "about",
                "About Flynn's Arcade",
                true,
                None::<&str>,
            )?;
            let sep = PredefinedMenuItem::separator(handle)?;
            let help = Submenu::with_items(
                handle,
                "Help",
                true,
                &[&game_documents, &github, &sep, &about],
            )?;
            #[cfg(target_os = "macos")]
            {
                // macOS convention: the first submenu is the app menu,
                // and quitting lives there as the predefined Quit item.
                // Clear Cache rides in the app menu, above Quit — the
                // macOS counterpart of its File-menu place elsewhere.
                let quit = PredefinedMenuItem::quit(handle, None::<&str>)?;
                let app_sep = PredefinedMenuItem::separator(handle)?;
                let app_menu = Submenu::with_items(
                    handle,
                    "Flynn's Arcade",
                    true,
                    &[&clear_cache, &app_sep, &quit],
                )?;
                Menu::with_items(handle, &[&app_menu, &help])
            }
            #[cfg(not(target_os = "macos"))]
            {
                let exit = MenuItem::with_id(handle, "exit", "Exit", true, None::<&str>)?;
                let file_sep = PredefinedMenuItem::separator(handle)?;
                let file = Submenu::with_items(
                    handle,
                    "File",
                    true,
                    &[&clear_cache, &file_sep, &exit],
                )?;
                Menu::with_items(handle, &[&file, &help])
            }
        })
        .on_menu_event(|app, event| match event.id().as_ref() {
            "exit" => app.exit(0),
            "clear-cache" => request_clear_cache(app),
            "game-documents" => navigate_main(app, "/documents.html"),
            "github" => {
                // The arcade's source repository; like every external
                // link in this app, it opens in the OS browser.
                let _ = app.opener().open_url(
                    "https://github.com/caeled/flynns-arcade-app",
                    None::<&str>,
                );
            }
            "about" => navigate_main(app, "/about.html"),
            _ => {}
        })
        .on_page_load(|webview, payload| {
            // If this launch consumed a Clear Cache flag, say so on
            // the page once it has loaded (once only).
            if webview.label() == "main"
                && matches!(payload.event(), PageLoadEvent::Finished)
                && CACHE_JUST_CLEARED.swap(false, std::sync::atomic::Ordering::SeqCst)
            {
                let _ = webview.eval(CACHE_CLEARED_NOTICE_JS);
            }
        })
        .setup(|app| {
            let handle = app.handle().clone();
            // Clear Cache lands here, before any window — and therefore
            // any webview — exists in this process: if the previous run
            // left the flag, delete the profile's cache directories now
            // (nothing holds their files yet) and arm the notice that
            // tells the user it happened once the shelf shows.
            if consume_clear_cache_flag(&handle) {
                CACHE_JUST_CLEARED.store(true, std::sync::atomic::Ordering::SeqCst);
            }
            let nav_handle = handle.clone();
            let new_window_handle = handle.clone();
            WebviewWindowBuilder::new(&handle, "main", WebviewUrl::App("splash.html".into()))
                .title("Flynn's Arcade")
                .inner_size(1280.0, 900.0)
                .on_navigation(move |url| {
                    if is_app_url(url) {
                        // The arcade itself: the interface and the
                        // bundled game and lesson pages stay inside
                        // the window.
                        true
                    } else if url.scheme() == "http" || url.scheme() == "https" {
                        // An outside page (a lesson's Wikipedia link,
                        // the collection's own site, ...): hand it to
                        // the OS default browser instead of navigating
                        // the arcade away — the window has no back
                        // button, so an outside page would strand the
                        // player. The arcade stays exactly where it was.
                        let _ = nav_handle.opener().open_url(url.as_str(), None::<&str>);
                        false
                    } else {
                        true
                    }
                })
                .on_new_window(move |url, _features| {
                    // An external link that asks for its own window
                    // gets the same treatment: the OS browser, never
                    // a second in-app window.
                    if url.scheme() == "http" || url.scheme() == "https" {
                        let _ = new_window_handle
                            .opener()
                            .open_url(url.as_str(), None::<&str>);
                    }
                    tauri::webview::NewWindowResponse::Deny
                })
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Flynn's Arcade");
}
