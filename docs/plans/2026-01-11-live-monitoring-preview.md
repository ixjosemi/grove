# Live File Monitoring + Smart Preview

## Decisiones de Diseño

- **Feedback visual**: Indicador `*` amarillo + "Live" en barra de estado
- **Preview contenido**: Texto (25 líneas) + metadata (tamaño, permisos, fecha)
- **Duración indicador**: 5 segundos
- **Cache**: Hasta que watcher detecte cambio

## Arquitectura

```
┌─────────────────────────────────────────────────────────┐
│                      main.rs                            │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐ │
│  │ Event Loop  │◄───│   Watcher   │    │   Preview   │ │
│  │ (crossterm) │    │  (notify)   │    │   Cache     │ │
│  └──────┬──────┘    └──────┬──────┘    └──────┬──────┘ │
│         │                  │                  │        │
│         ▼                  ▼                  ▼        │
│  ┌─────────────────────────────────────────────────┐   │
│  │                    App State                     │   │
│  │  - recent_changes: HashMap<PathBuf, Instant>    │   │
│  │  - preview_cache: HashMap<PathBuf, PreviewData> │   │
│  │  - watcher_rx: Receiver<PathBuf>                │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

## Archivos Nuevos

### src/watcher.rs (~60 líneas)

```rust
pub struct FileWatcher {
    watcher: RecommendedWatcher,
}

pub fn start_watcher(root: &Path) -> Result<(FileWatcher, Receiver<PathBuf>)>
```

- Observa directorio root recursivamente
- Debounce de 300ms (notify nativo)
- Filtra: `.swp`, `.swo`, `~`, `.#*`, `.DS_Store`
- Envía solo path afectado via mpsc channel

### src/preview.rs (~100 líneas)

```rust
pub struct PreviewData {
    pub path: PathBuf,
    pub content: PreviewContent,
    pub metadata: PreviewMetadata,
    pub cached_at: Instant,
}

pub struct PreviewMetadata {
    pub size: u64,
    pub modified: SystemTime,
    pub permissions: u32,
}

pub enum PreviewContent {
    Text(Vec<String>),        // Primeras 25 líneas
    Directory(Vec<DirEntry>), // Hijos: nombre + is_dir
    Binary,                   // Detectado via NULL bytes
    TooLarge,                 // > 50KB
    Error(String),
}

pub fn generate_preview(path: &Path) -> Result<PreviewData>
```

### src/ui/preview.rs (~80 líneas)

```rust
pub fn render_preview_overlay(frame: &mut Frame, app: &App)
```

Overlay centrado (60% width, 70% height):
- Header con nombre de archivo
- Metadata: tamaño, fecha modificación, permisos
- Contenido con scroll indicator

## Modificaciones

### src/app.rs (+40 líneas)

```rust
pub struct App {
    // ... existente ...
    pub watcher_rx: Option<Receiver<PathBuf>>,
    pub recent_changes: HashMap<PathBuf, Instant>,
    pub preview_cache: HashMap<PathBuf, PreviewData>,
    pub show_preview: bool,
    pub preview_scroll: usize,
}

impl App {
    pub fn check_watcher(&mut self)
    pub fn cleanup_old_changes(&mut self)
    pub fn is_recently_changed(&self, path: &Path) -> bool
    pub fn invalidate_preview_cache(&mut self, path: &Path)
}
```

### src/main.rs (+30 líneas)

- Iniciar watcher en main()
- Llamar check_watcher() y cleanup_old_changes() en loop
- Keybindings: Space (toggle preview), PageUp/PageDown (scroll)

### src/ui/tree.rs (+15 líneas)

- Indicador `*` amarillo para archivos recién modificados
- Texto "Live" en help bar cuando watcher activo

### src/ui/mod.rs (+3 líneas)

- Añadir `mod preview;`
- Llamar render_preview_overlay() si show_preview

### Cargo.toml

- Quitar `tokio` (no usado)
- `notify` ya está incluido

## Keybindings Nuevos

| Tecla | Acción |
|-------|--------|
| Space | Toggle preview overlay |
| PageUp | Scroll preview arriba (5 líneas) |
| PageDown | Scroll preview abajo (5 líneas) |

## Estimación

~330 líneas nuevas de código.
