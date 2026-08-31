# Luna — Windows Virtual Secretary

## 1. Project Purpose
Luna is a personal, background-triggered voice assistant for Windows. Activated via a global hotkey, Luna features a "fiery" secretary personality and manages desktop tasks seamlessly. She controls Microsoft Office applications (Word, PowerPoint, Outlook), performs file searches, checks weather and news, executes web searches, and handles dictionary lookups.

## 2. Architecture Overview
Luna follows an event-driven pipeline designed for speed, privacy, and low latency:
- **Global Hotkey:** Triggers push-to-talk audio recording via `global-hotkey`.
- **Speech-to-Text (STT):** `whisper-rs` (bindings for `whisper.cpp`) transcribes spoken audio on-demand (loaded into memory only when active to conserve RAM).
- **Brain Router:** Evaluates system connectivity and foreground application context to route prompts to either Groq Cloud API or local Ollama instances.
- **Skill Dispatch:** Parses intent and dispatches tasks to specific skill handlers (Office bridge, Everything CLI file search, Weather, News, Web Search, Dictionary).
- **Text-to-Speech (TTS):** Kokoro-82M via `ort` (ONNX Runtime) synthesizes output audio responses.
- **Memory Store:** SQLite (`rusqlite`) stores conversation logs and extracted long-term user facts.

## 3. Online/Offline Strategy
Luna dynamically selects its LLM engine based on internet connectivity and GPU load:
- **Online:** Routes requests to Groq Cloud API (OpenAI-compatible) for ultra-fast, high-quality responses.
- **Offline:** Calls local Ollama HTTP API (`http://localhost:11434`):
  - **Standard Mode (Idle / Web Browser active):** Uses `Llama 3 8B Q4_K_M` for rich, nuanced responses.
  - **GPU-Heavy Mode (e.g. Minecraft or games active in foreground):** Uses a lighter model such as `Phi-4-mini` or `Qwen 3B/4B` to prevent frame drops and resource contention.
- **Personality Consistency:** The same system prompt defining Luna's fiery secretary personality is applied across all models.

## 4. Folder Structure

```
luna/
├── src-tauri/                         # Rust Tauri backend
│   ├── Cargo.toml                     # Rust package manifest & dependencies
│   ├── tauri.conf.json                # Tauri configuration file
│   └── src/
│       ├── main.rs                    # Tauri app entrypoint and setup
│       ├── commands/
│       │   └── mod.rs                 # Exposed Tauri commands for frontend IPC
│       ├── stt/
│       │   └── mod.rs                 # whisper-rs wrapper (on-demand STT loading)
│       ├── tts/
│       │   └── mod.rs                 # Kokoro-82M ONNX wrapper via ort crate
│       ├── brain/
│       │   ├── mod.rs                 # LLM router logic (online vs local)
│       │   ├── cloud.rs               # Groq Cloud API client
│       │   ├── local.rs               # Ollama local API client
│       │   └── connectivity.rs        # Short-timeout ping connectivity check
│       ├── context/
│       │   └── mod.rs                 # Foreground app detection (GPU-heavy application check)
│       ├── skills/
│       │   ├── mod.rs                 # Skill registry & dispatcher
│       │   ├── office_bridge.rs       # Python subprocess bridge for pywin32 Office automation
│       │   ├── file_search.rs         # es.exe (Everything CLI) search wrapper
│       │   ├── weather.rs             # Open-Meteo REST client
│       │   ├── news.rs                # RSS feed fetcher and parser
│       │   ├── web_search.rs          # Python subprocess bridge for ddgs search
│       │   └── dictionary.rs          # dictionaryapi.dev API client
│       ├── memory/
│       │   └── mod.rs                 # SQLite connection & schema for history and facts
│       └── config/
│           └── mod.rs                 # App settings (API keys, hotkeys, paths)
├── src/                               # Frontend web assets (vanilla JS/HTML/CSS)
│   ├── index.html                     # UI layout: chat panel, thinking process, status bar
│   ├── style.css                      # Modern dark theme styles
│   └── main.js                        # Frontend IPC logic
├── python-bridge/                     # Python scripts for COM and DDGS bridges
│   ├── office_control.py              # pywin32 COM automation for Word, PowerPoint, Outlook
│   ├── web_search.py                  # duckduckgo-search (ddgs) integration
│   └── requirements.txt               # Dependencies: pywin32, duckduckgo-search
├── .gitignore                         # Ignore targets, pycache, .env
└── README.md                          # Project overview and architecture documentation
```

## 5. Free-Tool Inventory
- **Groq Free Tier:** Fast online cloud LLM endpoint.
- **Ollama:** Local model runner for offline intelligence.
- **Kokoro-82M:** High quality, lightweight local text-to-speech model.
- **whisper.cpp / whisper-rs:** Efficient C++ speech recognition bindings.
- **Open-Meteo API:** Free weather forecast API requiring no API key.
- **RSS Feeds:** Standard web protocols for news retrieval.
- **duckduckgo-search (ddgs):** Python package for free web search results.
- **dictionaryapi.dev:** Free public dictionary API for word definitions.
- **Everything / es.exe:** Lightning-fast Windows file search CLI tool by voidtools.

## 6. Setup Instructions

### Prerequisites
1. **Rust Toolchain:** Install Rust via [rustup.rs](https://rustup.rs/).
2. **Tauri CLI:** Install Tauri CLI (`cargo install tauri-cli` or `npm install -g @tauri-apps/cli`).
3. **Python 3.10+:** Ensure Python is installed and added to PATH. Install bridge dependencies:
   ```bash
   pip install -r python-bridge/requirements.txt
   ```
4. **Ollama:** Download and install [Ollama](https://ollama.com/). Pull required models:
   ```bash
   ollama pull llama3:8b
   ollama pull phi4-mini
   ```
5. **Everything CLI (`es.exe`):** Download and install Voidtools Everything and `es.exe` CLI, placing `es.exe` in your system PATH.

### Running Luna
In the project directory, run:
```bash
cargo tauri dev
```
