# aw-watcher-screenshot

## Overview

- This is a plugin for ActivityWatch.
- This plugin captures screenshots of the user's screen by monitors and sends them to the ActivityWatch server.
- After capturing the screenshot, it uses dHash and ocr to analyze the image.
- Captured screenshots are stored in the local storage or S3.
- It also provides VLM functionality.

## Dev Requirements

- Current Dev OS: Fedora
- Using rust
- rustc 1.92.0 (ded5c06cf 2025-12-08)
- cargo 1.92.0 (344c4567c 2025-10-21)

## Principle

- DO NOT change the edition of rust in Cargo.toml
