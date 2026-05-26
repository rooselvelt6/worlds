# 🌍 WORLDS — Motor 3D de Mundos Infinitos

**Generación procedural · Rust WASM + Three.js · Audio sintetizado · Terreno configurable en vivo**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Three.js](https://img.shields.io/badge/Three.js-r128-blue.svg)](https://threejs.org)
[![Leptos](https://img.shields.io/badge/Leptos-0.8-purple.svg)](https://leptos.dev)
[![Tailwind](https://img.shields.io/badge/Tailwind-4-06B6D4.svg)](https://tailwindcss.com)

WORLDS genera mundos 3D infinitos con terreno procedural FBM, zonas temáticas, personaje personalizable, ciclo día/noche, partículas ambientales y audio sintetizado. Todo corre en el navegador sin dependencias externas.

---

## Tech Stack

| Capa | Tecnología |
|------|-----------|
| Motor 3D | Three.js (WebGL2) |
| Lógica de terreno | Rust → WASM (wasm-bindgen) |
| UI | Leptos 0.8 CSR + Tailwind 4 |
| Servidor | Axum (Rust) |
| Audio | Web Audio API (síntesis 100%) |
| Post-procesado | UnrealBloomPass |

## Arquitectura

```
worlds/
├── client/                    # Motor Rust → Wasm
│   ├── src/
│   │   ├── engine/            # Núcleo del motor
│   │   │   ├── mod.rs         # Game loop + física + colisión
│   │   │   ├── terrain.rs     # Altura FBM, zonas, efectos, colores
│   │   │   ├── chunk.rs       # Generación de mallas + voxel 3D
│   │   │   ├── bridge.rs      # FFI → JavaScript
│   │   │   ├── audio.rs       # Síntesis de audio por zona
│   │   │   ├── controls.rs    # Teclado + mouse + gamepad
│   │   │   ├── camera.rs      # Cámara primera/tercera persona
│   │   │   ├── particles.rs   # Lluvia/Nieve/Insectos ambientales
│   │   │   ├── vegetation.rs  # Árboles, arbustos, rocas
│   │   │   ├── structures.rs  # Estructuras arquitectónicas
│   │   │   ├── minerals.rs    # Depósitos de minerales
│   │   │   ├── creatures.rs   # Criaturas procedurales + IA
│   │   │   ├── portals.rs     # Portales
│   │   │   ├── codex.rs       # Codex de criaturas
│   │   │   ├── achievements.rs# Logros
│   │   │   ├── inventory.rs   # Inventario + crafteo
│   │   │   ├── db.rs          # IndexedDB persistencia
│   │   │   ├── foam.rs        # Espuma de agua
│   │   │   └── waterfall.rs   # Cascadas
│   │   ├── math/              # Ruido FBM y funciones
│   │   ├── state/mod.rs       # Estado global, tipos
│   │   └── app.rs             # UI Leptos (menús deslizantes)
│   ├── three_bridge.js        # Render Three.js
│   ├── i18n/                  # Traducciones ES/EN
│   ├── manifest.json          # PWA manifest
│   └── service-worker.js      # Service worker
├── server/                    # Servidor Axum
│   └── assets/                # Frontend estático
└── shared/                    # Librería compartida
```

## Características

### 🏔️ Terreno Configurable

El terreno usa **FBM (Fractional Brownian Motion)** como función de ruido única, con parámetros ajustables en vivo:

- **Escala** (0.001–0.1): frecuencia del ruido
- **Amplitud** (0.5–20): altura máxima del terreno
- **Octavas** (1–10): detalle del ruido
- **Cañones**: tallado profundo con ondas sinusoidales
- **15 Zonas**: Forest, Plains, Desert, Tundra, Jungle, Volcanic, Ocean, Crystal, Cave, Lava, Fungus, Abyss, Storm, Aurora, Magma — cada una con color, altura y efectos únicos

### 🗺️ Terreno Voxel 3D Subterráneo

- **32 capas** de profundidad con bloqueo sólido
- **Cuevas, acuíferos, lava tubes, cavernas de hongo, geodas, dungeon rooms**
- LOD 3 niveles (32/8/2 capas)
- Iluminación por vóxel con antorchas + bloques emisivos
- Color blending superficie↔subterráneo
- **7 nuevos tipos de bloque**: Dirt, Stone, Wood, Leaves, Crystal, Lava Stone, Ice, Sand, Moss

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

### 🔊 Audio 100% Sintetizado

- Paisajes sonoros por zona
- Sonidos de pasos
- Clima dinámico

### 🌲 Vegetación 3D

- Árboles, arbustos, rocas, cactus, hongos, cristales, corales, algas, anémonas, esponjas
- Sway animado por viento
- Hasta 120 instancias por chunk

### 🦎 Criaturas Procedurales

- 16 tipos: Ciervo, Mono, Ave, Cristalino, Murciélago, Elemental de fuego, Serpiente, Oso polar, Zorro, Suricata, Pez, Cangrejo, Medusa, Mariposa, Pájaro, Luciérnaga
- Asignadas por bioma (terrestres, acuáticas, voladoras)
- IA con estados: idle, wander, flee, follow, eat
- Pathfinding A* sobre grid del terreno
- **Alimentar y domar**: clic derecho con fruta → domesticación
- Criaturas domadas siguen al jugador
- Movimiento sinusoidal + huida del jugador
- Descubrimiento vía Codex con clic derecho

### 🏛️ Estructuras Arquitectónicas

- Plazas, Murallas, Entradas de mazmorra, Torres, Ruinas, Arcos, Pilares, Cúpulas, Pirámides, Espirales de cristal, Cabañas de hongo, Obeliscos
- Variedad por bioma

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

- **Cascadas**: efecto visual con partículas en acantilados
- **Espuma**: foam en la línea de costa
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
| G | Ciclar clima |
| T | Activar/desactivar vuelo |
| B | Modo construcción |
| Click | Activar pointer lock |
| Click derecho | Examinar criatura / Recolectar fruta |

## Build & Deploy

```bash
# Requisitos: Rust + trunk
cargo install trunk

# Build WASM release
cd client && trunk build --release

# Deploy al servidor
cp dist/*.wasm dist/*.js ../server/assets/
# Actualizar hash en server/assets/index.html manualmente
# o con sed (el hash está en el nombre del archivo .wasm)

# Iniciar servidor
cd .. && cargo run --release -p worlds-server
```

## Desarrollo

```bash
# Build WASM + deploy rápido
cd client && trunk build --release && \
HASH=$(ls dist/worlds-app-*.wasm | sed 's/.*worlds-app-//;s/_bg.wasm//') && \
cp dist/*.wasm dist/*.js ../server/assets/ && \
sed -i "s/worlds-app-[a-f0-9]*/worlds-app-$HASH/g" ../server/assets/index.html

# Verificar compilación
cargo check -p worlds-app
```

## Licencia

MIT — ver [LICENSE](LICENSE) para detalles.

---

**WORLDS** — Genera mundos 3D infinitos en tu navegador.
