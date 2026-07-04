# APK Analyzer - Project Memory

## Project Overview
- **Type**: Tauri 2 + React + TypeScript + Rust desktop application
- **Purpose**: AI-powered APK file analysis tool for Android developers
- **Target**: Windows desktop (cross-platform: also builds for macOS/Linux)

## Tech Stack
- Frontend: React 18, TypeScript, Vite, Tailwind CSS, Zustand, Lucide icons
- Backend: Rust (stable 1.96+), Tauri 2
- APK parsing: Pure Rust (no external Java tools needed)
  - Binary Android XML decoder (custom implementation)
  - ZIP reader via `zip` crate
  - X.509 certificate parsing via `x509-parser` crate
  - DEX file parser (custom)
  - APK Signing Block parser (custom)

## Architecture
- Each analyzer implements `Analyzer` trait with `analyze(&self, apk: &mut ApkReader) -> Result<Output, String>`
- Analyzers are independent modules in `src-tauri/src/analyzers/`
- Data models shared between Rust and TypeScript via `serde`
- Tauri IPC commands in `src-tauri/src/commands/mod.rs`
- Progress updates via Tauri events

## Build Environment (macOS)
- Rust installed at: project-local `.cargo/` and `.rustup/` (CARGO_HOME/RUSTUP_HOME in project dir)
- Node.js: `/Users/tianchao/.workbuddy/binaries/node/versions/22.22.2/bin/node`
- Use `cargo build -j 1` to avoid OOM kills on 16GB Mac

## Key Decisions
- Pure Rust parsing (no apktool/JADX) for zero external dependencies
- Tauri 2 (not Electron) for small binary size and native performance
- Feature-based module organization for easy extensibility

## Implemented Analyzers
1. Overview - basic app info, icons, ABIs, densities, languages
2. Manifest - full AndroidManifest.xml parsing
3. Permissions - classified database of 40+ known permissions
4. Components - activities/services/receivers/providers with intent filters
5. Resources - by-type, largest, duplicates, image stats
6. Native Libraries - grouped by ABI with compression info
7. DEX - multi-DEX, class/method/field counts, package hierarchy
8. Certificate - V1/V2/V3 detection, X.509 parsing, hash computation
9. Security - 8+ automated checks with scoring
10. AI Summary - tech stack detection (15+ frameworks), risk analysis

## Future Extensions (planned)
- AAB/APKM/XAPK support
- APK comparison
- ADB integration
- VirusTotal integration
- Plugin system
