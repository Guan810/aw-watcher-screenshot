# aw-watcher-screenshot

## Overview

- This is a plugin for ActivityWatch.
- This plugin captures screenshots of the user's screen by monitors and sends them to the ActivityWatch server.
- After capturing the screenshot, it uses dHash and ocr to analyze the image.
- Captured screenshots are stored in the local storage or S3.
- It also provides VLM functionality.

## Dev Requirements

- Current Dev OS: Windows 11
- Using rust
- rustc 1.91.1 (ed61e7d7e 2025-11-07)
- cargo 1.91.1 (ea2d97820 2025-10-10)

## Principle

- DO NOT change the edition of rust in Cargo.toml
