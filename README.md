# ordb (Organizado y Resolutor de base de Datos)

**`ordb`** is a two-tier system designed to organize, deduplicate, and classify massive volumes of historical files. It ensures zero data loss via binary deduplication and transactional state management.

For a detailed breakdown of the system components, see [`ARCHITECTURE.md`](ARCHITECTURE.md).

---

## Prerequisites

- **Python 3.8+**
- **Rust + Cargo** — install from [rustup.rs](https://rustup.rs/)
- **SQLite3** (used internally by the CLI)

---

## How to Run

The system has two components that must both be running: the Python AI Microservice and the Rust CLI.

### 1. Start the AI Microservice (`ordb-ai`)

```bash
cd ordb-ai

# Create and activate the virtual environment (first time only)
python -m venv venv

# Windows
.\venv\Scripts\activate
# Linux / Mac
source venv/bin/activate

# Install dependencies (first time only)
pip install -r requirements.txt

# Start the service
uvicorn main:app --reload
```

The service will be available at `http://localhost:8000`. Keep this terminal open.

### 2. Run the CLI (`ordb-cli`)

Open a new terminal:

```bash
cd ordb-cli

# Build (first time only)
cargo build --release

# Run
cargo run --release -- --source /path/to/source --dest /path/to/destination
```

Use `--help` to see all available flags (e.g. `--dry-run`, thread options).
