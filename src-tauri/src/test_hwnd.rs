
pub fn test(app: tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let hwnd = w.hwnd().unwrap();
        let _ = hwnd.0;
    }
}

