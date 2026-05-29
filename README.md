# WORLDS — Motor 3D de Mundos Infinitos

**Generacion procedural · Rust WASM + Three.js · Audio sintetizado · Terreno configurable en vivo**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Three.js](https://img.shields.io/badge/Three.js-r175-blue.svg)](https://threejs.org)
[![Leptos](https://img.shields.io/badge/Leptos-0.8-purple.svg)](https://leptos.dev)

WORLDS genera mundos 3D infinitos con terreno procedural FBM, texturizado por pendiente, zonas tematicas, personaje personalizable, ciclo dia/noche, particulas ambientales y audio sintetizado. Todo corre en el navegador.

---

## Estado del Desarrollo

| Fase | Estado |
|------|--------|
| ✅ F5 — Persistencia (IndexedDB) | Completado |
| ✅ F7 — Terreno Voxel 3D (Cuevas) | Completado |
| ✅ F8 — Ecosistemas Dinamicos | Completado |
| ✅ F9 — Criaturas con IA | Completado |
| ✅ F10 — Audio 3D Inmersivo | Completado |
| ✅ F11 — Portales (shader, fade, hub) | Completado |
| ✅ F13 — Hidrologia (rios, cascadas, oleaje) | Completado |
| ✅ F14 — Poderes Climaticos | Completado |
| ✅ F15 — Codex / Bestiario | Completado |
| ✅ F16 — Arquitectura & Civilizacion | Completado |
| ✅ F17 — Modding API (biomas JSON, blueprints, paletas) | Completado |
| ✅ F18 — Optimizacion & Pulido (LOD, frustum, PWA, i18n) | Completado |
| 🚀 F19 — Web Workers | Pendiente |
| 🚀 F20 — Mejoras Mobile | Pendiente |
| 🚀 F21 — Bosses | Pendiente |
| 🚀 F22 — Narrativa y Misiones | Pendiente |

## Tech Stack

| Capa | Tecnologia |
|------|-----------|
| Motor 3D | Three.js r175 (local, sin CDN) |
| Logica de terreno | Rust 2021 -> WASM (wasm-bindgen) |
| UI | Leptos 0.8 CSR |
| Servidor | Axum (Rust, con WebSocket) |
| Audio | Web Audio API (sintesis 100%) |
| Post-procesado | EffectComposer + SSRPass + SSAOPass + UnrealBloomPass |

## Arquitectura

```
worlds/
├── client/                    # Motor Rust -> Wasm
│   ├── src/
│   │   ├── engine/            # Nucleo del motor
│   │   │   ├── mod.rs         # Game loop + fisica + colision
│   │   │   ├── terrain.rs     # Altura FBM, zonas, colores, pendiente
│   │   │   ├── chunk.rs       # Malla superficie continua + LOD 3 niveles
│   │   │   ├── bridge.rs      # FFI -> JavaScript
│   │   │   ├── audio.rs       # Sintesis de audio 3D por zona
│   │   │   ├── controls.rs    # Teclado + mouse + gamepad
│   │   │   ├── particles.rs   # Lluvia/Nieve/Insectos
│   │   │   ├── vegetation.rs  # Arboles, arbustos, rocas
│   │   │   ├── structures.rs  # 12 tipos de estructuras
│   │   │   ├── minerals.rs    # Depositos de minerales
│   │   │   ├── creatures.rs   # 16 tipos de criaturas con IA
│   │   │   ├── portals.rs     # Portales entre mundos
│   │   │   ├── codex.rs       # Bestiario
│   │   │   ├── achievements.rs# Logros
│   │   │   ├── inventory.rs   # Inventario + crafteo
│   │   │   ├── db.rs          # IndexedDB persistencia
│   │   │   ├── foam.rs        # Espuma de agua
│   │   │   ├── waterfall.rs   # Cascadas
│   │   │   └── modding/       # Modding API (biomas, formulas, blueprints)
│   │   ├── math/              # Ruido FBM y formulas
│   │   ├── state/mod.rs       # Estado global
│   │   ├── i18n.rs            # Internacionalizacion
│   │   └── app.rs             # UI Leptos
│   ├── three_bridge.js        # Render Three.js + post-procesado
│   ├── i18n/                  # Traducciones ES/EN/FR/DE/JA
│   ├── manifest.json          # PWA manifest
│   └── service-worker.js      # Service worker v4
├── server/                    # Servidor Axum
│   └── assets/                # Frontend estatico + Three.js local
│       └── three/             # three.module.js + three.core.js
└── shared/                    # Libreria compartida Rust
```

## Características

### Superficie Continua (Mesh Suave)

El terreno dejó de ser voxel blocky para usar un **mesh de superficie continuo** generado por `compute_chunk_data_lod`:

- Cada chunk de 16x16 celdas muestrea altura en esquinas (17x17 puntos)
- Triangulación con winding CCW para normales correctas
- Sin caras +Y superficiales — el mesh es una sola capa que sigue la altura del terreno
- LOD 3 niveles: step 1 (16x16, 512 tris), step 2 (8x8, 128 tris), step 4 (4x4, 32 tris)
- Frustum culling por bounding sphere por chunk

### Texturizado por Pendiente (Slope Mapping)

Los colores y tiles se seleccionan según la inclinacion del terreno:

| Pendiente | Textura | Color |
|-----------|---------|-------|
| < 0.3 | Pasto / arena / zona | Verde / segun bioma |
| 0.3 - 0.6 | Tierra | Marron |
| > 0.6 | Piedra | Gris |
| > 85% altura max | Nieve | Blanco |

La pendiente se calcula como `sqrt(dzdx^2 + dzdy^2)` con diferencias centrales de las 4 esquinas de cada celda.

###  Terreno Configurable

El terreno usa **FBM (Fractional Brownian Motion)** como funcion de ruido unica, con parametros ajustables en vivo:

- **Escala** (0.001-0.1): frecuencia del ruido
- **Amplitud** (0.5-20): altura maxima del terreno
- **Octavas** (1-10): detalle del ruido
- **Canones**: tallado profundo con ondas sinusoidales
- **15 Zonas**: Forest, Plains, Desert, Tundra, Jungle, Volcanic, Ocean, Crystal, Cave, Lava, Fungus, Abyss, Storm, Aurora, Magma

###  Terreno Voxel 3D Subterraneo

- **32 capas** de profundidad con bloqueo solido
- Cuevas, acuiferos, lava tubes, cavernas de hongo, geodas, dungeon rooms
- LOD 3 niveles (32/8/2 capas)
- Iluminacion por voxel con antorchas + bloques emisivos
- Color blending superficie/subterraneo
- 9 tipos de bloque: Dirt, Stone, Wood, Leaves, Crystal, Lava Stone, Ice, Sand, Moss

### 💧 Agua Dinámica

- Nivel de agua configurable (−1.0 a 2.0)
- Ondas Gerstner (4 componentes) en tiempo real
- Opacidad y color ajustables

### 🧑 Personaje Personalizable

- **6 Presets**: Humano, Robot, Bestia, Fantasma, Teddy, Panda, Kraken
- **7 Esquemas de color** con paletas propias
- **Escala** (0.5×–1.5×)
- Animación de caminar/correr con brazos y piernas
- Ocultamiento automático en primera persona

### 🎥 Cámara

- **Primera persona**: altura de ojos, pitch/yaw directos
- **Tercera persona**: orbital, sigue al personaje
- Alternable en vivo con un clic

### 🌧️ Partículas

- **Lluvia**: 1200 gotas, caída rápida, color azul claro
- **Nieve**: 600 copos, caída lenta, blancos
- Encendido/apagado desde la UI

### ☀️ Ciclo Día/Noche

- Desactivado por defecto
- Velocidad ajustable (0 a 0.5)
- Sol en arco, cielo transiciona, estrellas aparecen de noche
- Cambio de color de luz y niebla

### 🌸 Estaciones y Ecosistemas

- **4 estaciones**: Primavera, Verano, Otoño, Invierno con ciclo automático
- Colores de follaje, floración y tamaño de vegetación cambian por estación
- **Crecimiento de árboles**: semilla → brote → joven → adulto con ticks cada ~10s
- **Frutos**: aparecen en árboles en verano/otoño, recolectables con clic derecho
- **Insectos de bioma**: mariposas (Forest/Jungle/Plains), pájaros (abiertos), luciérnagas (Cave/Fungus)
- **Frentes meteorológicos**: masas de lluvia que se desplazan por el mapa
- **Viento**: dirección y fuerza variables, afecta partículas y vegetación

### 🏃 Física y Colisiones

- **Gravedad** configurable (5–40)
- **Salto** ajustable (2–25)
- **Colisión horizontal**: el personaje no atraviesa montañas
- **Step-up automático**: sube escalones hasta `step_height`
- **Aceleración y fricción**: movimiento suave
- **Bloqueo contra bloques**: no atraviesa estructuras colocadas
- **Natación**: flotabilidad y gravedad reducida bajo agua
- **Vuelo**: sin gravedad, Space/Shift sube/baja

### 🔊 Audio 3D Inmersivo

- **Audio posicional 3D** con `PannerNode`: sonidos de criaturas, portales y eventos siguen la posición del oyente
- **Paisajes sonoros por zona**: ambiente base con frecuencia y volumen según bioma
- **Sistema musical dinámico**: capas de bajo (triangle) y pad (sine) con notas musicales por bioma, moduladas por altura, velocidad y hora del día
- **Eco en cuevas** con `ConvolverNode` + impulso procedural (reverberación en Cave/Abyss)
- Sonidos de pasos según superficie (tierra, piedra, pasto, arena, agua, nieve)
- Clima dinámico: lluvia graduada por intensidad, truenos
- Efectos: portales, curaciones, crafteo, rescate de criaturas

### 🌲 Vegetación 3D

- Árboles, arbustos, rocas, cactus, hongos, cristales, corales, algas, anémonas, esponjas
- Sway animado por viento
- Hasta 120 instancias por chunk

### 🦎 Criaturas Procedurales

- 16 tipos: Ciervo, Mono, Ave, Cristalino, Murciélago, Elemental de fuego, Serpiente, Oso polar, Zorro, Suricata, Pez, Cangrejo, Medusa, Mariposa, Pájaro, Luciérnaga
- Asignadas por bioma (terrestres, acuáticas, voladoras)
- IA con estados: idle, wander, flee, follow, eat
- Pathfinding A* sobre grid del terreno
- **Animaciones**: marcha de 4 patas en cuadrúpedos, aleteo en insectos/aves, pulso en elementales, nado en peces, vaivén en medusas
- **Alimentar y domar**: clic derecho con fruta → domesticación
- **Rescate**: criaturas ocultas en mazmorras, al rescatarlas se vuelven dóciles y dan minerales
- **Montura**: criaturas domadas tipo ciervo y oso son montables con tecla F, control WASD
- Criaturas domadas siguen al jugador
- Movimiento sinusoidal + huida del jugador
- Descubrimiento vía Codex con clic derecho

### 🏛️ Estructuras Arquitectónicas

- Plazas, Murallas, Entradas de mazmorra, Torres, Ruinas, Arcos, Pilares, Cúpulas, Pirámides, Espirales de cristal, Cabañas de hongo, Obeliscos
- Variedad por bioma: tipos, tamaño y color según zona
- **Puentes**: generados automáticamente sobre ríos entre caminos de estructuras
- **Murallas**: perímetros defensivos con almenas
- **Dungeons subterráneos**: salas de 7x3x7 bajo estructuras grandes (Plaza, Pirámide, Torre, Cúpula) con tesoro central y hongos luminosos

### 🎮 Multijugador

- WebSocket vía servidor Axum
- Chat en tiempo real
- Jugadores remotos visibles como cápsulas con nombre

### 📖 Codex y Logros

- **Codex**: registro de criaturas descubiertas por bioma, con nombre y zona
- **Logros**: sistema de logros desbloqueables por acciones (domar, descubrir, explorar)

### 💾 Persistencia

- Save/Load automático y manual vía IndexedDB
- 3 ranuras de guardado con nombre y timestamp
- Persiste: posición, waypoints, seed, parámetros, inventario, codex, logros, bloques colocados

### 🏭 Inventario y Crafteo

- Inventario con recolección de frutos y minerales
- Sistema de crafteo básico
- Consumibles: fruta para alimentar criaturas

### 🌊 Hidrología

- **Ríos**: cauces naturales tallados con noise sinusoidal, rellenos de agua superficial (bloques BLK_WATER) en canales
- **Cascadas**: efecto visual con partículas en acantilados + sonido espacial 3D
- **Espuma**: foam en la línea de costa
- **Burbujas**: partículas ascendentes bajo el agua
- **Flora acuática**: algas, corales, kelp en zonas marinas
- **Oleaje**: vertex displacement en el agua

## UI — Menús Deslizantes

Interfaz de 3 columnas con botones de acción directa. Cada botón abre un panel contextual con sliders, presets y opciones:

| Columna | Botones |
|---------|---------|
| 1 | 🌱 Semilla · 🏃 Física · 🎥 Cámara · ☀️ Día/Noche |
| 2 | ⚙️ Escala · 📏 Amplitud · 🔢 Octavas · 💧 Agua · 🌍 Zona · 🏔️ Cañones · 🌧️ Partículas |
| 3 | 🧑 Personaje · 🎨 Color · 📐 Tamaño |

## Controles

| Tecla | Acción |
|-------|--------|
| W/S | Adelante/atrás |
| A/D | Izquierda/derecha |
| ESPACIO | Saltar / Subir (vuelo/natación) |
| SHIFT | Bajar (vuelo/natación) |
| Q/E | Rotar cámara |
| F | Montar/desmontar criatura domada |
| R | Teletransportarse por portal cercano |
| G | Ciclar clima |
| T | Activar/desactivar vuelo |
| B | Modo construcción |
| Click | Activar pointer lock |
| Click derecho | Examinar criatura / Recolectar fruta |

## Build & Deploy

```bash
# Requisitos: Rust + trunk + Node.js (para Three.js)
cargo install trunk

# Build WASM release (sin wasm-bindgen-rayon por compatibilidad)
cd client && trunk build --release --no-default-features

# Deploy al servidor
HASH=$(ls dist/worlds-app-*.wasm | sed 's/.*worlds-app-//;s/_bg.wasm//')
cp dist/worlds-app-$HASH.js dist/worlds-app-${HASH}_bg.wasm ../server/assets/
cp dist/index.html dist/three_bridge.js ../server/assets/
sed -i "s/worlds-app-[a-f0-9]*/worlds-app-$HASH/g" ../server/assets/index.html

# Descargar Three.js local (necesario para Brave/Chrome sin CDN)
# mkdir -p ../server/assets/three
# curl -o ../server/assets/three/three.module.js https://unpkg.com/three@0.175.0/build/three.module.js
# curl -o ../server/assets/three/three.core.js https://unpkg.com/three@0.175.0/build/three.core.js
# (postprocessing y shaders tambien necesarios)

# Iniciar servidor
cd .. && cargo run --release -p worlds-server
```

## Desarrollo

```bash
# Build WASM + deploy rapido
cd client && trunk build --release --no-default-features && \
HASH=$(ls dist/worlds-app-*.wasm | sed 's/.*worlds-app-//;s/_bg.wasm//') && \
cp dist/worlds-app-$HASH.js dist/worlds-app-${HASH}_bg.wasm dist/index.html dist/three_bridge.js ../server/assets/ && \
sed -i "s/worlds-app-[a-f0-9]*/worlds-app-$HASH/g" ../server/assets/index.html

# Verificar compilacion
cargo check -p worlds-app
```

## Licencia

MIT — ver [LICENSE](LICENSE) para detalles.

---

**WORLDS** — Genera mundos 3D infinitos en tu navegador.
