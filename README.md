# 🌍 WORLDS — Motor 3D de Mundos Infinitos

**Generación procedural · Rust WASM + Three.js · Audio sintetizado · 27 fórmulas · 15 biomas**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Three.js](https://img.shields.io/badge/Three.js-r128-blue.svg)](https://threejs.org)
[![Leptos](https://img.shields.io/badge/Leptos-0.8-purple.svg)](https://leptos.dev)
[![Tailwind](https://img.shields.io/badge/Tailwind-4-06B6D4.svg)](https://tailwindcss.com)

WORLDS transforma fórmulas matemáticas en mundos 3D infinitos y navegables, con paisajes sonoros únicos por bioma, vegetación procedural, estructuras arquitectónicas, minerales con emisión, cuevas talladas por ruido 3D, ciclo día/noche, y un sistema de descubrimiento. Todo corre en el navegador sin dependencias externas de assets.

---

## Tech Stack

| Capa | Tecnología |
|------|-----------|
| Motor 3D | Three.js (WebGL2) + GLSL shaders |
| Lógica de terreno | Rust → WASM (wasm-bindgen) |
| UI | Leptos 0.8 CSR + Tailwind 4 |
| Servidor | Axum (Rust) |
| Audio | Web Audio API (síntesis 100%, 0 archivos) |
| Post-procesado | UnrealBloomPass |

## Arquitectura

```
worlds/
├── client/                    # Motor Rust → Wasm
│   ├── src/
│   │   ├── engine/            # Núcleo del motor (13 módulos)
│   │   │   ├── mod.rs         # Game loop + chunk generation
│   │   │   ├── terrain.rs     # Altura, zonas, efectos, colores
│   │   │   ├── chunk.rs       # Generación de mallas (vértices, normales, índices)
│   │   │   ├── bridge.rs      # 32 funciones FFI → JavaScript
│   │   │   ├── audio.rs       # Orquestador de audio/clima/tinte
│   │   │   ├── controls.rs    # Teclado + mouse pointer lock
│   │   │   ├── camera.rs      # Cámara primera persona
│   │   │   ├── joystick.rs    # Joystick táctil para móvil
│   │   │   ├── minimap.rs     # Minimapa Canvas 2D
│   │   │   ├── particles.rs   # Partículas ambientales
│   │   │   ├── vegetation.rs  # Vegetación procedural
│   │   │   ├── structures.rs  # Estructuras arquitectónicas
│   │   │   └── minerals.rs    # Depósitos de minerales
│   │   ├── math/builtins.rs   # 30+ funciones de ruido/fórmulas
│   │   ├── state/mod.rs       # Estado global, tipos, paletas
│   │   └── app.rs             # UI Leptos (glassmorphism, 5 tabs)
│   └── three_bridge.js        # Render Three.js + audio + post (1600+ líneas)
├── server/                    # Servidor Axum
│   └── assets/                # Frontend servido estáticamente
└── shared/                    # Librería compartida Rust
```

## Características

### 🧮 27 Fórmulas Matemáticas

Cada una genera un universo único con paleta de color propia (23 gradientes):

`FBM · Perlin · Simplex · Voronoi · Mandelbrot · Sierpinski · Julia · Tetrahedron · Cube · Sphere · Menger · Vortex · Ice · Wave · Spiral · Hexagonal · RidgedMF · DomainWarp · Hybrid · Plasma · Cellular · Strange Attractor · Worley · Marble · Terrazas · Erosion · Thermal`

6 fórmulas incluyen sliders de parámetros ajustables en vivo.

### 🌿 15 Biomas

Cada bioma define su propia altura, color, vegetación, estructuras, minerales, partículas, clima, sonido ambiente, tinte lumínico y viñeta:

`Forest · Plains · Desert · Tundra · Jungle · Volcanic · Ocean · Crystal · Cave · Lava · Fungus · Abyss · Storm · Aurora · Magma`

### 🔊 Audio 100% Sintetizado

- **15 paisajes sonoros** por bioma (viento, bosque, océano, cueva, tormenta, lava, cristal, aurora, etc.) vía Web Audio API
- **Sonidos de pasos** al caminar en tierra
- **Efectos UI** al cambiar fórmula o zona
- **Clima dinámico**: lluvia, nieve, tormenta, polvo, ceniza con niebla adaptativa (color + densidad)
- Slider de volumen maestro

### 🎨 Gráficos

- **Sombras dinámicas** PCFSoft 2048×2048
- **Agua con shader GLSL**: ondas (3 frecuencias), gradiente profundo→somero, foam, shimmer, ciclo día/noche
- **Bloom** con UnrealBloomPass, intensidad por zona
- **Ciclo día/noche** continuo (sol en arco, luz ambiental cambia, cielo transiciona)
- **Nubes + estrellas + auroras** procedurales
- **Viñeta adaptativa** por bioma (0.5 desierto → 0.95 abismo)
- **Tinte de bioma** en luz ambiental
- **Partículas ambientales**: hojas, arena, nieve, chispas, esporas, cristales

### 🌲 Vegetación 3D

8 tipos por bioma, hasta 120 instancias por chunk vía `InstancedMesh`:

`Tree · Bush · Rock · Cactus · Mushroom · Crystal · DeadTree · Flower`

- Densidad 0.0–0.7 según zona
- Validación de pendiente y altura de agua
- Sway animado por viento (sinusoidal, fase aleatoria)
- Sombras dinámicas

### 🏛️ Estructuras

10 tipos arquitectónicos con geometrías mergeadas:

`Hut · Tower · Ruins · Arch · Pillar · Dome · Pyramid · CrystalSpire · MushroomHut · Obelisk`

- Colocación contextual: 0–3 por chunk, área plana validada, altura sobre agua
- Por bioma: pirámides en desierto, domos en tundra, torres en tormenta
- **Estructuras ocultas** descubribles por proximidad

### ⛏️ Cuevas y Minerales

- **Cañones/cuevas 3D**: tallados con FBM + sinusoidal en zona Cave, con pilares naturales
- **Roca subterránea**: transición de color progresiva bajo el nivel del agua
- **8 minerales con emisión**: esmeralda, zafiro, cobre, cuarzo, amatista, rubí, topacio, perla

### 📦 Exportación

- **OBJ**: descarga de escena completa con normales
- **STL binario**: descarga con triángulos indexados
- **Screenshots**: F12 captura PNG con metadata
- **Seed**: guardado en localStorage (tecla H)

### 🎮 Jugabilidad

- **Waypoints**: tecla T marca posición, M remueve, contador en HUD
- **Descubrimiento de biomas**: notificación al visitar cada zona
- **Estructuras ocultas**: detección por proximidad (≤5m)

## Controles

| Tecla | Acción |
|-------|--------|
| W/S | Adelante/atrás |
| A/D | Izquierda/derecha |
| ESPACIO | Saltar / Subir (vuelo) |
| SHIFT | Agacharse / Bajar (vuelo) |
| Q/E | Rotar cámara |
| C | Alternar modo observador orbital |
| T | Marcar waypoint |
| M | Remover último waypoint |
| G | Exportar OBJ |
| H | Guardar seed en localStorage |
| F12 | Capturar screenshot |
| Click | Activar pointer lock |

## Quick Start

```bash
# Requisitos: Rust + wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Compilar todo
cargo build --release

# Build WASM
wasm-pack build --target web --out-dir pkg client

# Copiar al servidor
cp client/pkg/worlds_app_bg.wasm server/assets/
cp client/pkg/worlds_app.js server/assets/
cp client/three_bridge.js server/assets/

# Iniciar servidor
cargo run --release -p worlds-server

# Abrir
open http://localhost:3000
```

## Desarrollo

```bash
# Build WASM + deploy rápido
wasm-pack build --target web --out-dir pkg client && \
cp client/pkg/worlds_app_bg.wasm server/assets/ && \
cp client/pkg/worlds_app.js server/assets/ && \
cp client/three_bridge.js server/assets/

# Verificar compilación Rust
cargo check -p worlds-app
```

## Licencia

MIT — ver [LICENSE](LICENSE) para detalles.

---

**WORLDS** — Genera mundos 3D infinitos en tu navegador.
