<p align="center">
  
# ReAmped
### A modern audio player focused on **visuals**, **performance**, and **extensibility**
  
<img width="1910" height="1032" alt="2026-03-11_16-15-57" src="demo/reamped.gif" />

Lightweight music player with real-time visualizers and other nice things

</p>

<p align="center">

![GitHub release](https://img.shields.io/github/v/release/XxAlexplosivoxX/ReAmped?style=for-the-badge)
![Platform](https://img.shields.io/badge/platform-linux%20%7C%20windows%20%7C%20android%20%28mayybe%20no%29-blue?style=for-the-badge)
![Rust](https://img.shields.io/badge/built%20with-rust-orange?style=for-the-badge)
![License](https://img.shields.io/github/license/XxAlexplosivoxX/ReAmped?style=for-the-badge)
<a href="https://github.com/XxAlexplosivoxX/ReAmped/releases">
  <img src="https://img.shields.io/badge/Download-Latest%20Release-2ea44f?style=for-the-badge&logo=github">
</a>
</p>

---

# Overview

**ReAmped** is a modern music player designed around **real-time audio visualization**.

It combines:

- Smooth music playback  
- Reactive visualizers  
- A modular plugin system -- _**under development**_

All built with **performance and extensibility in mind**.

---

# Preview

<p align="center">
<img width="1910" height="1032" alt="2026-03-11_16-15-57" src="demo/reamped2.gif" />
<img width="1910" height="1032" alt="2026-03-11_16-16-09" src="https://github.com/user-attachments/assets/42d3e3f6-39bd-4ab8-8690-1e2ef0be51ac" />
<img width="1910" height="1032" alt="2026-03-11_16-21-31" src="https://github.com/user-attachments/assets/f0dbb2b6-fad0-4e4d-ac6c-2a2125e4d6a0" />
<img width="1910" height="1032" alt="2026-03-11_16-21-49" src="https://github.com/user-attachments/assets/55b0accc-b003-4995-8a82-8e287a94711b" />
<img width="1910" height="1032" alt="2026-03-11_16-22-26" src="https://github.com/user-attachments/assets/b31a0156-fa40-491d-bf51-6d16a94e97fc" />
<img width="1910" height="1032" alt="2026-03-11_16-22-40" src="https://github.com/user-attachments/assets/2cc30219-f8ec-4bfd-b864-5bb72fe7b08f" />
<img width="1910" height="1032" alt="2026-03-11_16-22-55" src="https://github.com/user-attachments/assets/3b140ac4-c4b5-4235-be94-2c4c41543c72" />
<img width="1910" height="1032" alt="2026-03-11_16-23-44" src="https://github.com/user-attachments/assets/a017f90f-de33-494b-b51e-b53f2552faaa" />

</p>

---

# Download

Precompiled binaries are available in **GitHub Releases**.

[**Download here**](https://github.com/XxAlexplosivoxX/ReAmped/releases)

Supported platforms:

<table>
  <thead>
    <tr>
      <td>OS</td>
      <td>it runs?</td>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>Linux</td>
      <td>Yeah, it's on the releases as a precompiled binary</td>
    </tr>
    <tr>
      <td>Windows</td>
      <td>Sure, it's on the releases as a precompiled binary</td>
    </tr>
    <tr>
      <td>Android</td>
      <td>🗣️🗣️ Hell naah ❌❌❌❌ <i>(only if core-player is correctly functional in Android maybe)</i></td>
    </tr>
  </tbody>
</table>

No compilation required.

Just download and run.

---

# Features

## Audio Playback

- Fast audio playback engine
- Playlist support
- Shuffle / repeat / repeat-one modes
- Library scanning
- Volume control

---

# Architecture
<div align=center>
  <img src="demo/reamped_architecture.png">
</div>

---

## Audio Meters 
>Under development, but i think make it like a DAW or something

---

## Visualizers

ReAmped renders **audio visualizations in real time**.

Examples include:

- Spectrum analyzer
- Beat reactive visuals
- Waveform displays
- Dynamic UI color themes

---

## Plugin System
>Under Development

# Contributing to ReAmped

Thank you for your interest in contributing to ReAmped.

ReAmped is an experimental audio player focused on performance, real-time audio visualization. The project aims to provide a modern and extensible audio playback environment with minimal overhead and a modular architecture.

Contributions of all kinds are welcome.


## Ways to Contribute

You can contribute in many ways, including:

- fixing bugs
- improving performance
- creating visualizers
- improving UI layout
- suggesting new features
- improving code readability
- reporting issues

Even small improvements are appreciated.


## Development Focus

ReAmped is built with several main areas of development.


### Audio Core

The audio core is responsible for:

- playback
- plugin processing
- audio analysis
- feeding visualizers

Important rules:

- avoid allocations inside the audio processing thread
- keep audio processing deterministic
- prefer simple and predictable DSP code


### Visualizers

Visualizers receive analyzed audio data and render graphics in real time.


### User Interface

The UI is built using egui.

Contributions here may include:

- layout improvements
- new UI components
- improved responsiveness
- visual polish
- customizable themes
- better meter displays


## Android Support

Android is currently NOT supported.

The core audio engine (player-core) relies on the cpal crate for audio output.  
On Android, cpal attempts to use the AAudio backend.

At the moment this configuration does not compile reliably, which prevents building the project for Android.

Until the upstream issues with cpal and AAudio are resolved, an official Android version will not be developed.

Community experiments are welcome, but Android support is not planned until the backend situation improves.


## Pull Request Guidelines

When submitting a pull request:

- keep changes focused and minimal
- avoid mixing unrelated changes
- ensure the project builds successfully
- explain what the change does and why

Small and well-scoped pull requests are easier to review.


## Code Style

General guidelines:

- keep modules simple and readable
- avoid unnecessary abstractions
- prefer explicit code over clever code
- keep audio processing fast and predictable


## Feature Requests

If you want to suggest a feature:

1. open an issue
2. describe the idea clearly
3. explain the use case

Discussion is always welcome.


## Final Notes

ReAmped is still evolving and the architecture may change as the project grows.

Thanks for your interest in contributing.
