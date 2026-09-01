use gtk::prelude::*;

pub fn restart_app() {
    ic_utils::app::restart_app();
}


pub fn read_blocking(path: &str) -> bool {
    const MAX: u64 = 8 * 1024 * 1024;
    std::fs::metadata(path).map(|m| m.len() <= MAX).unwrap_or(false)
}

pub fn open_with_system(path: &std::path::Path) {
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(path).spawn();

    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(path).spawn();

    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd")
        .args(["/c", "start", ""])
        .arg(path)
        .spawn();
}

pub fn select_folder<F>(parent_win: &gtk::Window, title: &str, callback: F)
where
    F: Fn(std::path::PathBuf) + 'static,
{
    #[cfg(target_os = "windows")]
    {
        let title = title.to_string();
        let (tx, mut rx) = tokio::sync::mpsc::channel::<std::path::PathBuf>(1);

        gtk::glib::spawn_future_local(async move {
            if let Some(path) = rx.recv().await {
                callback(path);
            }
        });

        std::thread::spawn(move || {
            let res = rfd::FileDialog::new().set_title(&title).pick_folder();
            if let Some(path) = res {
                let _ = tx.blocking_send(path);
            }
        });
    }
    #[cfg(not(target_os = "windows"))]
    {
        let dialog = gtk::FileDialog::builder().title(title).build();
        let dialog_clone = dialog.clone();
        dialog.select_folder(Some(parent_win), gtk::gio::Cancellable::NONE, move |res| {
            let _keep_alive = dialog_clone;
            if let Ok(file) = res {
                if let Some(path) = file.path() {
                    callback(path);
                }
            }
        });
    }
}
