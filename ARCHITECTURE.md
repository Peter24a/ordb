# Architecture Map / Mapa de Arquitectura

## General Overview / Resumen General
**`ordb`** (Organizado y Resolutor de base de Datos) is a two-tier system designed to organize, deduplicate, and classify massive volumes of historical files. It ensures zero data loss via binary deduplication and transactional state management.
**`ordb`** es un sistema de dos niveles diseñado para organizar, deduplicar y clasificar volúmenes masivos de archivos históricos. Asegura que no haya pérdida de datos mediante deduplicación binaria y gestión transaccional del estado.

---

## English

### System Components

1. **`ordb-cli` (Rust)**
   The core engine responsible for file operations. It acts locally on the filesystem.
   - **`main.rs` & `cli.rs`**: Entry point and CLI argument parsing (defines options like directories, threads, dry-runs).
   - **`db.rs` & `schema.sql`**: SQLite database interactions. Tracks the state of each file natively to allow transaction resumes and prevent redundant processing.
   - **`scanner.rs`**: Fast filesystem scanning, reading files and computing SHA-256 hashes to find duplicates.
   - **`metadata.rs`**: Extraction of embedded metadata (EXIF for images/videos, ID3 for audio, dates, etc.).
   - **`enrichment.rs`**: Projection logic. Calculates the final destination structured path (using tiered templates like `Images/Year/Month_Name/Category`) based on metadata. Handles name collisions gracefully.
   - **`phases.rs`**: Execution coordination. Structures the workload into distinct phases: Scan, Enrich, Deduplicate, Stage, Commit/Rollback.
   - **`api_client.rs`**: Communication layer module to interact with the AI Microservice for classifying unrecognized files.

2. **`ordb-ai` (Python)**
   An intelligent microservice providing semantic zero-shot classification capabilities.
   - Designed to run independently, exposing a REST/gRPC API.
   - Analyzes images or documents using AI/ML models to assign appropriate taxonomy tags when structured metadata is insufficient.
   - Queried via the Rust CLI's API client during the enrichment phase.

---

## Español

### Componentes del Sistema

1. **`ordb-cli` (Rust)**
   El motor central responsable de las operaciones de archivos. Actúa localmente en el sistema de archivos.
   - **`main.rs` y `cli.rs`**: Punto de entrada y análisis de argumentos CLI (define opciones como directorios, hilos, ejecuciones de prueba "dry-run").
   - **`db.rs` y `schema.sql`**: Interacciones con la base de datos SQLite. Rastrea el estado de cada archivo de forma nativa para permitir reanudar transacciones y evitar procesamientos redundantes.
   - **`scanner.rs`**: Escaneo rápido del sistema de archivos, lectura de archivos y cálculo de hashes SHA-256 para encontrar duplicados.
   - **`metadata.rs`**: Extracción de metadatos incrustados (EXIF para imágenes/videos, ID3 para audio, fechas, etc.).
   - **`enrichment.rs`**: Lógica de proyección. Calcula la estructura de carpetas de destino final (usando plantillas escalonadas como `Imagenes/Año/Mes_Nombre/Categoría`) basándose en metadatos. Maneja colisiones de nombres.
   - **`phases.rs`**: Coordinación de ejecución. Estructura la carga de trabajo en fases separadas: Escaneo, Enriquecimiento, Deduplicación, Preparación, Confirmación/Reversión.
   - **`api_client.rs`**: Módulo de capa de comunicación para interactuar con el microservicio de Inteligencia Artificial para clasificar archivos no reconocidos.

2. **`ordb-ai` (Python)**
   Un microservicio inteligente que provee capacidades semánticas de clasificación tipo zero-shot.
   - Diseñado para correr de forma independiente, exponiendo una API REST/gRPC.
   - Analiza imágenes o documentos utilizando modelos de Inteligencia Artificial (AI/ML) para asignar etiquetas y categorías cuando los metadatos estructurados no son suficientes.
   - Es consultado a través del cliente API de la CLI en Rust durante la fase de enriquecimiento.
