use eframe::egui;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

fn main() -> Result<(), eframe::Error> {
    let rt = Runtime::new().expect("Failed to create Tokio runtime");

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(500.0, 300.0)),
        ..Default::default()
    };

    eframe::run_native(
        "YouTube Downloader",
        options,
        Box::new(|_cc| Box::new(YoutubeDownloader::new(rt))),
    )
}

#[derive(Default)]
struct YoutubeDownloader {
    url: String,
    save_path: String,
    is_audio_only: bool,
    progress: f32,
    rt: Option<Runtime>,
    status_message: String,
    progress_rx: Option<mpsc::UnboundedReceiver<(f32, String)>>,
}

impl eframe::App for YoutubeDownloader {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 檢查並處理進度更新
        if let Some(rx) = &mut self.progress_rx {
            while let Ok((progress, status)) = rx.try_recv() {
                self.progress = progress;
                self.status_message = status;
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("YouTube Downloader");

            ui.horizontal(|ui| {
                ui.label("YouTube URL:");
                ui.text_edit_singleline(&mut self.url);
            });

            ui.horizontal(|ui| {
                ui.label("Save Path:");
                ui.text_edit_singleline(&mut self.save_path);
                if ui.button("Browse").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.save_path = path.display().to_string();
                    }
                }
            });

            ui.checkbox(&mut self.is_audio_only, "audio only(MP3)");

            ui.add(egui::ProgressBar::new(self.progress));

            if !self.status_message.is_empty() {
                ui.label(&self.status_message);
            }

            if ui.button("Download").clicked() && !self.url.is_empty() {
                self.status_message = "Downloading...".to_string();
                self.progress = 0.0;
                self.start_download(ctx);
                ctx.request_repaint();
            }
        });
    }
}

impl YoutubeDownloader {
    fn new(rt: Runtime) -> Self {
        Self {
            rt: Some(rt),
            ..Default::default()
        }
    }

    fn start_download(&mut self, ctx: &egui::Context) {
        if self.url.is_empty() {
            self.status_message = "Please enter URL".to_string();
            return;
        }

        if self.save_path.is_empty() {
            self.status_message = "Please select save location".to_string();
            return;
        }

        let url = self.url.clone();
        let path = PathBuf::from(&self.save_path);
        let is_audio = self.is_audio_only;

        // 創建 channel
        let (progress_tx, progress_rx) = mpsc::unbounded_channel();
        self.progress_rx = Some(progress_rx);

        if let Some(rt) = &self.rt {
            let progress_tx = progress_tx.clone();

            rt.spawn(async move {
                let mut cmd = Command::new("/opt/homebrew/bin/yt-dlp");
                cmd.arg(&url)
                    .arg("-P")
                    .arg(&path)
                    .arg("--format")
                    .arg(if is_audio { "bestaudio" } else { "best" })
                    .arg("--progress-template")
                    .arg("download:[%(progress.downloaded_bytes)s/%(progress.total_bytes)s]")
                    .arg("--no-playlist")
                    .arg("--no-warnings")
                    .arg("--no-check-certificate")
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());

                if is_audio {
                    cmd.arg("-x")
                        .arg("--audio-format")
                        .arg("mp3")
                        .arg("--audio-quality")
                        .arg("0");
                }

                let mut child = match cmd.spawn() {
                    Ok(child) => child,
                    Err(e) => {
                        let _ = progress_tx.send((0.0, format!("Error: {}", e)));
                        return;
                    }
                };

                let stdout = child.stdout.take().unwrap();
                use tokio::io::{AsyncBufReadExt, BufReader};
                let mut reader = BufReader::new(stdout).lines();

                while let Ok(Some(line)) = reader.next_line().await {
                    if line.contains("download:") {
                        if let Some(percent) = parse_progress(&line) {
                            let _ = progress_tx
                                .send((percent, format!("Downloading... {:.1}%", percent * 100.0)));
                        }
                    }
                }

                match child.wait().await {
                    Ok(status) if status.success() => {
                        let _ = progress_tx.send((1.0, "Download completed!".to_string()));
                    }
                    Ok(_) => {
                        let _ = progress_tx.send((0.0, "Download failed".to_string()));
                    }
                    Err(e) => {
                        let _ = progress_tx.send((0.0, format!("Error: {}", e)));
                    }
                }
            });
        }
    }
}

fn parse_progress(line: &str) -> Option<f32> {
    if let (Some(start), Some(end)) = (line.find('['), line.find(']')) {
        let content = &line[start + 1..end];
        if let Some(separator) = content.find('/') {
            let current = content[..separator].parse::<f64>().ok()?;
            let total = content[separator + 1..].parse::<f64>().ok()?;
            return Some((current / total) as f32);
        }
    }
    None
}
