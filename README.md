# ordb (Organizado y Resolutor de base de Datos)

**`ordb`** is a two-tier system designed to organize, deduplicate, and classify massive volumes of historical files. It ensures zero data loss via binary deduplication and transactional state management.
**`ordb`** es un sistema de dos niveles diseñado para organizar, deduplicar y clasificar volúmenes masivos de archivos históricos. Asegura que no haya pérdida de datos mediante deduplicación binaria y gestión transaccional del estado.

For a detailed breakdown of the system components, please see [`ARCHITECTURE.md`](ARCHITECTURE.md).
Para un desglose detallado de los componentes del sistema, por favor consulta [`ARCHITECTURE.md`](ARCHITECTURE.md).

---

## 🚀 How to Run / Cómo Ejecutar

The system consists of two main components: the Rust CLI (`ordb-cli`) and the Python AI Microservice (`ordb-ai`). You must run both parts to utilize the full processing and AI classification pipeline.
El sistema consta de dos componentes principales: la CLI de Rust (`ordb-cli`) y el Microservicio de IA en Python (`ordb-ai`). Debes ejecutar ambas partes para utilizar todo el pipeline de procesamiento y clasificación por IA.

### 1. Python AI Microservice (`ordb-ai`)

This service must be running for the Rust CLI to successfully query semantic classifications.
Este servicio debe estar en ejecución para que la CLI de Rust pueda consultar clasificaciones semánticas con éxito.

**EN:**
1. Navigate to the `ordb-ai` directory:
   ```bash
   cd ordb-ai
   ```
2. Create and activate a virtual environment (recommended):
   ```bash
   python -m venv venv
   # On Windows:
   .\venv\Scripts\activate
   # On Linux/Mac:
   source venv/bin/activate
   ```
3. Install dependencies:
   ```bash
   pip install -r requirements.txt
   ```
4. Start the microservice:
   ```bash
   python main.py
   ```

**ES:**
1. Navega al directorio `ordb-ai`:
   ```bash
   cd ordb-ai
   ```
2. Crea y activa un entorno virtual (recomendado):
   ```bash
   python -m venv venv
   # En Windows:
   .\venv\Scripts\activate
   # En Linux/Mac:
   source venv/bin/activate
   ```
3. Instala las dependencias:
   ```bash
   pip install -r requirements.txt
   ```
4. Inicia el microservicio:
   ```bash
   python main.py
   ```

---

### 2. Rust CLI (`ordb-cli`)

This is the core engine that scans, deduplicates, and organizes your files.
Este es el motor central que escanea, deduplica y organiza tus archivos.

**EN:**
1. Ensure you have Rust and Cargo installed ([rustup.rs](https://rustup.rs/)).
2. Navigate to the `ordb-cli` directory:
   ```bash
   cd ordb-cli
   ```
3. Build the project in release mode for maximum performance:
   ```bash
   cargo build --release
   ```
4. Run the CLI tool with the required arguments (e.g., source and destination directories):
   ```bash
   # Run the executable directly
   ../target/release/ordb-cli --source /path/to/source --dest /path/to/destination

   # Or run via Cargo
   cargo run --release -- --source /path/to/source --dest /path/to/destination
   ```
   *Tip: Use `--help` to see all available flags, such as `--dry-run` or thread options.*

**ES:**
1. Asegúrate de tener Rust y Cargo instalados ([rustup.rs](https://rustup.rs/)).
2. Navega al directorio `ordb-cli`:
   ```bash
   cd ordb-cli
   ```
3. Compila el proyecto en modo release para obtener el máximo rendimiento:
   ```bash
   cargo build --release
   ```
4. Ejecuta la herramienta CLI con los argumentos requeridos (ej. directorios de origen y destino):
   ```bash
   # Ejecuta el binario directamente
   ../target/release/ordb-cli --source /ruta/al/origen --dest /ruta/al/destino

   # O ejecuta a través de Cargo
   cargo run --release -- --source /ruta/al/origen --dest /ruta/al/destino
   ```
   *Consejo: Usa `--help` para ver todas las opciones disponibles, como `--dry-run` o las opciones de hilos.*

---

## 🛠 Prerequisites / Requisitos Previos
- **Rust** (latest stable version) / **Rust** (última versión estable)
- **Python 3.8+** / **Python 3.8+**
- **SQLite3** (used internally by the CLI) / **SQLite3** (usado internamente por la CLI)
