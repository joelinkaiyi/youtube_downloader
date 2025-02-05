# YouTube Downloader

A simple YouTube video/audio downloader built with Rust, using egui for the GUI and yt-dlp as the backend.

## Features

- Download YouTube videos in best quality
- Extract audio from videos (MP3 format)
- User-friendly GUI interface
- Real-time download progress
- Custom save location selection

## Prerequisites

Before running this application, make sure you have:

1. Rust installed (https://rustup.rs/)
2. yt-dlp installed
   ```bash
   # For macOS (using Homebrew)
   brew install yt-dlp
   
   # For Linux
   sudo curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /usr/local/bin/yt-dlp
   sudo chmod a+rx /usr/local/bin/yt-dlp
   ```

## Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/youtube_downloader.git
   cd youtube_downloader
   ```

2. Build and run:
   ```bash
   cargo run
   ```

## Usage

1. Launch the application
2. Paste a YouTube URL into the URL field
3. Select a save location using the "Browse" button
4. Toggle "audio only" if you want to extract audio (MP3)
5. Click "Download" to start downloading
6. Monitor the progress bar and status messages

## Dependencies

- `eframe`: GUI framework
- `tokio`: Async runtime
- `rfd`: File dialog
- `yt-dlp`: YouTube download backend
