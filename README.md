# APK Analyzer

AI-Powered APK Analyzer — a modern desktop application for inspecting Android APK files with AI-powered analysis and developer insights.

## Features

### Core Analysis
- **Overview** — App icon, name, package, version, SDK levels, APK size, ABIs, languages, densities, security flags
- **Manifest** — Full AndroidManifest.xml parsing (activities, services, receivers, providers, intent filters, permissions, features, queries, meta-data)
- **Permissions** — Classified by protection level (Normal/Dangerous/Signature/Special) with risk assessment, descriptions, and usage recommendations
- **Components** — Statistics and detailed view of all components, exported component detection, intent filter expansion
- **Resources** — By-type breakdown, largest resources, duplicate detection, image format statistics
- **Native Libraries** — Grouped by ABI, with architecture info, sizes, and compression ratios
- **DEX** — Multi-DEX support, class/method/field counts, package hierarchy, largest packages
- **Certificate** — V1/V2/V3 signature detection, SHA1/SHA256/MD5 hashes, validity, debug cert detection
- **Security** — Automated security checks with scoring (0-100), issue categorization by severity, actionable recommendations
- **AI Summary** — Technology stack detection (Flutter, React Native, Unity, Kotlin, Compose, etc.), architecture guess, risk analysis, performance/packaging suggestions, permission review

### UX
- Drag-and-drop APK loading
- Recent files history
- Global search across all analysis results
- Export to JSON, Markdown, HTML, CSV
- Dark/Light theme
- Resizable panels
- Progress indicators with cancellation support
- Copy any field to clipboard

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Frontend | React 18 + TypeScript + Vite + Tailwind CSS |
| Backend | Rust + Tauri 2 |
| APK Parsing | Pure Rust (zip, binary XML decoder, DEX parser, signing block parser) |
| State Management | Zustand |
| Icons | Lucide React |

### Architecture

The application follows a clean, modular architecture:

```
src-tauri/src/
├── parser/           # Low-level APK parsing
│   ├── mod.rs        # ApkReader (ZIP archive reader)
│   ├── axml.rs       # Binary Android XML decoder
│   ├── signing.rs    # APK Signing Block / X.509 certificate parser
│   ├── resources.rs  # resources.arsc parser
│   └── dex.rs        # DEX file parser
├── analyzers/        # Independent analysis modules
│   ├── mod.rs        # Analyzer trait definition
│   ├── overview.rs   # App overview
│   ├── manifest.rs   # Manifest parsing
│   ├── permissions.rs# Permission analysis
│   ├── components.rs # Component analysis
│   ├── resources.rs  # Resource analysis
│   ├── native_libs.rs# Native library analysis
│   ├── dex.rs        # DEX analysis
│   ├── certificate.rs# Certificate analysis
│   ├── security.rs   # Security checks
│   └── ai_summary.rs # AI summary generation
├── models/           # Data models (shared with frontend via serde)
├── commands/         # Tauri IPC commands
├── export/           # Report export (JSON/MD/HTML/CSV)
└── utils/            # Utilities (permission DB, tech detector, recent files)
```

Each analyzer implements the `Analyzer` trait:

```rust
pub trait Analyzer {
    type Output: Serialize + DeserializeOwned + Send + 'static;
    fn name(&self) -> &'static str;
    fn analyze(&self, apk: &mut ApkReader) -> Result<Self::Output, String>;
}
```

## Building

### Prerequisites

1. **Rust** (stable): Install via [rustup](https://rustup.rs/)
2. **Node.js** 18+ and npm
3. **Windows-specific**: Visual Studio C++ Build Tools (for Tauri)

### Install Dependencies

```bash
npm install
```

### Development Mode

```bash
npm run tauri:dev
```

### Production Build

```bash
npm run tauri:build
```

This produces:
- **Windows**: `.msi` installer and `.exe` in `src-tauri/target/release/bundle/`
- **macOS**: `.dmg` and `.app` in `src-tauri/target/release/bundle/`
- **Linux**: `.deb` and `.AppImage` in `src-tauri/target/release/bundle/`

## Adding a New Analyzer

1. Create a new module in `src-tauri/src/analyzers/`:

```rust
use crate::parser::ApkReader;
use crate::models::my_analysis::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MyAnalysisResult {
    pub field1: String,
    pub field2: u32,
}

pub struct MyAnalyzer;

impl super::Analyzer for MyAnalyzer {
    type Output = MyAnalysisResult;

    fn name(&self) -> &'static str { "my_analyzer" }

    fn analyze(&self, apk: &mut ApkReader) -> Result<Self::Output, String> {
        // Parse data from APK
        Ok(MyAnalysisResult { field1: "value".into(), field2: 42 })
    }
}
```

2. Add the model in `src-tauri/src/models/`
3. Register in `src-tauri/src/analyzers/mod.rs`
4. Add to `ApkAnalysis` struct in `src-tauri/src/models/analysis.rs`
5. Call the analyzer in `src-tauri/src/commands/mod.rs` (`analyze_apk` function)
6. Add the TypeScript type in `src/types/index.ts`
7. Create a React page in `src/pages/`
8. Add navigation item in `src/components/layout/Sidebar.tsx`

## No External Dependencies

The APK parser is implemented entirely in Rust — no Java, no apktool, no JADX required for core analysis. This means:
- Fast startup
- Small binary size
- No JVM overhead
- Works completely offline

## License

MIT
