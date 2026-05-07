# WORLDS - Motor 3D de Mundos Infinitos

Un motor de generación procedural de mundos 3D en el navegador con Rust + WebAssembly + Three.js.

![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)
![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)
![Three.js](https://img.shields.io/badge/Three.js-r128-blue.svg)

## Características

- **Generación Procedural**: Terreno infinito generado con ruido FBM (Fractional Brownian Motion)
- **Múltiples Biomas**: Forest, Plains, Desert, Tundra, Jungle, Volcanic, Ocean, Lava
- **Personalizable**: Cambia seed, frecuencia, octavas, altura, nivel de agua en tiempo real
- **Colores Customizables**: Customize cada bioma con tu paleta de colores
- **Navegación 3D**: Movete libremente por el mundo generado
- **Rendimiento**: 60 FPS con chunks dinámicos

## Tech Stack

| Componente | Tecnología |
|------------|-------------|
| Motor 3D | Three.js |
| WASM | Rust + wasm-bindgen |
| UI | TailwindCSS |
| Servidor | Axum (Rust) |
| Ruido | noise-rs (Simplex/Perlin) |

## Arquitectura

```
/worlds
├── client/          # Motor voxel en Rust → Wasm
├── server/         # Servidor Axum
├── shared/         # Generador de mundos (Rust)
└── server/assets/  # Frontend + assets
```

## Quick Start

```bash
# Compilar
cargo build --release

# Correr servidor
cargo run --release -p worlds-server

# Abrir浏览器
open http://localhost:3000
```

## Controles

| Tecla | Acción |
|-------|--------|
| W/S | Mover adelante/atrás |
| A/D | Mover izquierda/derecha |
| ESPACIO | Subir |
| SHIFT | Bajar |
| Q/E | Girar cámara |
| R | Regenerar mundo |

### Panel de Configuración

- **Seed**: Semilla del mundo (1-9999)
- **Frecuencia**: Escala del ruido
- **Detalle**: Número de octavas (1-8)
- **Altura**: Altura máxima montañas
- **Nivel Agua**: Altura del agua
- **Dist Render**: Cuántos chunks cargar

## API

### Generar Chunk

```bash
GET /api/chunk/{x}/{y}/{z}
```

Respuesta:
```json
{
  "x": 0, "y": 0, "z": 0,
  "blocks": [...],
  "heightmap": [...],
  "biome": "forest"
}
```

## Desarrollo

```bash
# Desarrollo con hot reload
cargo run --release -p worlds-server

# Build Wasm
wasm-pack build --target web --out-dir server/assets/client client
```

## Contributing

1. Fork el repo
2. Crea tu branch (`git checkout -b feature/amazing`)
3. Commit tus cambios (`git commit -m 'Add amazing feature'`)
4. Push a GitHub (`git push origin main`)
5. Abre un Pull Request

## Licencia

MIT License - ve el archivo LICENSE para detalles.

---

**WORLDS** - Generates infinite 3D worlds in your browser.