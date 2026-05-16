# 🌍 WORLDS — Plan de Transformación Visual y de Jugabilidad

## Estado Actual (Base)
- Motor Rust WASM + Three.js (Leptos UI)
- 19 fórmulas matemáticas de terreno
- 10 zonas/biomas
- Controles: WASD + mouse pointer lock
- Cámara: primera persona
- UI: paneles negros con sliders básicos

---

## Visión General

Transformar WORLDS en una experiencia visualmente impactante, con interfaz poética, controles tipo gamepad/joystick, minimapa, nuevos universos de generación y efectos cinematográficos.

---

## FASE 1: Interfaz Renacida (UI/UX)
**Objetivo**: Paneles bellos, metáforas poéticas, sin funciones repetidas.

### Cambios en app.rs (Leptos UI):
- [ ] **Glassmorphism profundo**: Paneles con `backdrop-blur-xl`, bordes luminosos, sombras con color dinámico según fórmula activa
- [ ] **Metáforas poéticas** en etiquetas:
  - "Semilla del Mundo" → seed
  - "Gravedad Onírica" → amplitude
  - "Horizonte" → render distance
  - "Frecuencia Vital" → scale
  - "Océano Interior" → water level
  - "Armonía" → hue shift
  - "Intensidad" → saturation
  - "Luminosidad" → lightness
- [ ] **Tabs deslizantes**: Mundo | Fórmula | Color | Control (organización tipo acordeón)
- [ ] **Animaciones**: sliders con glow progresivo, botones con ripple, transiciones suaves
- [ ] **Modo Simple/Avanzado**: toggle que ocuestra sliders complejos
- [ ] **Tipografía**: títulos en Orbitron, datos en JetBrains Mono, etiquetas en Inter
- [ ] **Iconos SVG animados** en vez de emojis planos

**Archivos**: `client/src/app.rs`, `client/index.html` (CSS)

---

## FASE 2: Joystick Virtual + Control Táctil + Gamepad
**Objetivo**: Navegar por cualquier superficie con control intuitivo.

### Nuevo archivo:
- [ ] `client/src/engine/joystick.rs` — Lógica de joystick virtual + gamepad

### Cambios:
- [ ] **Joystick circular** táctil (Canvas 2D + pointer events):
  - Zona activa en esquina inferior izquierda
  - Círculo base semitransparente + círculo interno que sigue el dedo
  - Vector de dirección se traduce a movimiento (WASD analógico)
- [ ] **Botones de acción** tipo gamepad (A/B/X/Y):
  - A = Saltar, B = Sprint, X = Fly/Walk, Y = Interactuar
- [ ] **D-pad como alternativa** táctil
- [ ] **Gamepad API**: soporte para mandos USB/bluetooth
- [ ] **Control híbrido**: joystick visible en móvil, oculto en desktop (o toggle)
- [ ] **Modo Surface Follow**: el joystick adapta la cámara para seguir cualquier pendiente
- [ ] Toque doble en canvas: activa/desactiva joystick

**Archivos**: `client/src/engine/joystick.rs` (nuevo), `client/src/engine/mod.rs`, `client/src/engine/controls.rs`, `client/src/app.rs`

---

## FASE 3: Minimap + Brújula Celeste
**Objetivo**: Ver el mapa del terreno alrededor y orientación.

### Cambios en app.rs:
- [ ] **Minimapa circular** (esquina inferior derecha, 180px):
  - Renderizado en Canvas 2D desde Rust (no Three.js, más ligero)
  - Vista cenital del terreno circundante (radio = render_distance × chunk_size)
  - Coloreado por bioma real (usa `get_zone()`)
  - Punto brillante = jugador, línea = dirección de mirada
  - Grid de chunks sutil
- [ ] **Brújula holográfica** tipo disco flotante (parte superior):
  - N/E/S/W en fuente mono
  - Dirección actual se ilumina con glow
- [ ] **Altímetro vertical analógico**: barra vertical con gradiente, aguja marcando altura actual
- [ ] **Coordenadas** elegantes en esquina superior izquierda (formato "X: 124 · Z: 89 · ALT: 23.4")

### Renderizado:
- El minimapa se dibuja en un `<canvas>` separado desde Rust con `web_sys::CanvasRenderingContext2d`
- Se actualiza cada ~100ms (no cada frame)
- Usa `get_height()` y `get_zone()` ya existentes

**Archivos**: `client/src/app.rs`, `client/src/engine/minimap.rs` (nuevo)

---

## FASE 4: Universo de Fórmulas Expandido
**Objetivo**: Más variedad, blending entre fórmulas, mutación procedural.

### Nuevas fórmulas en math/builtins.rs:
- [ ] **Plasma**: `sin(x * freq + z * freq * 0.5 + t) * cos(z * freq - x * freq * 0.3)`
- [ ] **Cellular Automata**: ruido basado en reglas CA 1D aplicadas en 2D
- [ ] **Strange Attractor (Clifford)**: `x_{n+1} = sin(a·y_n) + c·cos(a·x_n)`, etc.
- [ ] **Worley Noise**: distancia cellular euclidiana con 3 puntos por celda
- [ ] **Marble**: `sin(fbm(x,z) * freq + fbm(x*2, z*2) * 0.5)`
- [ ] **Terrazas**: cuantización de FBM en escalones: `floor(h * levels) / levels`
- [ ] **Erosion**: simulación simplificada de erosión hidráulica
- [ ] **Thermal**: mezcla de ruido térmico + FBM

### Blending de fórmulas:
- [ ] `BlendA` = slider `mezcla` (0-1) entre fórmula actual y secundaria seleccionada
- [ ] `BlendB` = slider `peso` para ponderar

### Mutación procedural:
- [ ] Cada chunk puede usar variante ligeramente diferente según `seed + cx + cz`
- [ ] Parámetros: `(fórmula_base ± pequeña variación)`

### Nuevos biomas:
- [ ] **Fungus**: colores púrpura/verde neón, formas redondeadas
- [ ] **Abyss**: oscuro profundo, altura negativa constante, columnas
- [ ] **Storm**: alto contraste, crestas afiladas, colores gris/azul tormenta
- [ ] **Aurora**: verde/azul brillante, ondas suaves
- [ ] **Magma**: rojo/naranja brillante, fisuras luminosas

### Nuevo enum + actualizaciones:
- [ ] `client/src/state/mod.rs`: añadir variantes a `FormulaType`
- [ ] `client/src/math/builtins.rs`: implementar funciones
- [ ] `client/src/engine/terrain.rs`: añadir al match de `get_height()` y colores
- [ ] `client/src/app.rs`: botones para nuevas fórmulas y biomas

**Archivos**: `client/src/math/builtins.rs`, `client/src/state/mod.rs`, `client/src/engine/terrain.rs`, `client/src/app.rs`

---

## FASE 5: Cielo Vivo + Efectos Visuales
**Objetivo**: Entorno inmersivo con ciclo día/noche y atmósfera dinámica.

### Cambios en three_bridge.js y engine:
- [ ] **Skybox procedural**:
  - Gradiente de cielo según bioma: zenith → horizon
  - Nubes simples (sprites o shader)
  - Estrellas en modo noche
  - Auroras para zona Aurora
- [ ] **Ciclo día/noche**:
  - Tiempo continuo 0-1 (o manual con slider)
  - Sol se mueve en arco (luz direccional sigue)
  - Luz ambiental cambia intensidad y color (cálido día, azul noche)
  - Cielo transiciona suavemente
  - Luna visible en noche
- [ ] **Niebla volumétrica adaptativa**:
  - Color de niebla según bioma
  - Densidad varía con altura (más densa en valles)
- [ ] **Agua semitransparente** (nuevo mesh plano en y=water_level con transparencia y animación de ondas)
- [ ] **Partículas ambientales**:
  - Forest: hojas verdes cayendo
  - Desert: arena levantada
  - Tundra: nieve cayendo
  - Volcanic/Lava: chispas ascendentes
  - Crystal: cristales brillantes flotando
  - Jungle: pétalos/frutos
  - Fungus: esporas brillantes

### Implementación:
- Partículas con `THREE.Points` y `PointsMaterial`
- Datos de posición/color generados desde Rust
- Ciclo día/noche controlado desde engine mod.rs

**Archivos**: `client/three_bridge.js`, `client/src/engine/mod.rs`, `client/src/engine/particles.rs` (nuevo)

---

## FASE 6: Post-procesado Cinematográfico
**Objetivo**: Efectos visuales avanzados con pasada de post-proceso.

### Cambios en three_bridge.js:
- [ ] **UnrealBloomPass** para resplandor:
  - Intensidad según zona (fuerte en Crystal/Lava, sutil en Forest)
- [ ] **Color grading**:
  - LUT por bioma (tibio, frío, saturado, desaturado)
- [ ] **Vignette** dinámico:
  - Más notable en cuevas/cavernas
- [ ] **Depth of field**:
  - Desenfoque suave en distancias lejanas

### Dependencias:
- Importar `three/addons/postprocessing/EffectComposer.js`
- Importar `three/addons/postprocessing/RenderPass.js`
- Importar `three/addons/postprocessing/UnrealBloomPass.js`
- Importar `three/addons/shaders/LuminosityHighPassShader.js`
- Importar `three/addons/shaders/CopyShader.js`

**Archivos**: `client/three_bridge.js`

---

## FASE 7: Exploración Aumentada
**Objetivo**: Herramientas para explorar, marcar y capturar el mundo.

### Cambios en app.rs y engine:
- [ ] **Waypoints**: marcar posición actual con nombre (tecla M)
  - Se muestran en minimapa como puntos de color
  - Lista en panel lateral
- [ ] **Modo Observador** (tecla C):
  - Cámara libre orbital alrededor del jugador
  - Scroll para zoom in/out
  - Ideal para apreciar paisajes
- [ ] **Fotos Instantáneas** (tecla F12):
  - Captura el canvas actual
  - Descarga PNG con metadata (seed, fórmula, zona, coordenadas)
- [ ] **Rastro de luz**: trail brillante detrás del jugador (opcional, toggle)
- [ ] **Modo Noche**: toggle rápido para oscurecer entorno y ver estrellas/luz de cristales

**Archivos**: `client/src/app.rs`, `client/src/engine/mod.rs`, `client/three_bridge.js`

---

## Resumen de Archivos a Modificar/Crear

### Archivos existentes (modificar):
| Archivo | Cambios |
|---------|---------|
| `client/src/app.rs` | F1 (UI), F3 (minimap), F4 (nuevos botones), F7 |
| `client/src/engine/mod.rs` | F2 (joystick integración), F5 (cielo), F7 |
| `client/src/engine/controls.rs` | F2 (gamepad, joystick input) |
| `client/src/engine/terrain.rs` | F4 (nuevas fórmulas, biomas, colores) |
| `client/src/state/mod.rs` | F4 (nuevos FormulaType, biomas) |
| `client/src/math/builtins.rs` | F4 (nuevas funciones de ruido) |
| `client/three_bridge.js` | F5 (cielo, partículas), F6 (post-procesado) |
| `client/index.html` | F1 (CSS), F6 (imports three addons) |

### Archivos nuevos (crear):
| Archivo | Propósito |
|---------|-----------|
| `client/src/engine/joystick.rs` | F2: Lógica de joystick virtual + gamepad |
| `client/src/engine/minimap.rs` | F3: Renderizado de minimapa en Canvas 2D |
| `client/src/engine/particles.rs` | F5: Sistema de partículas ambientales |

---

## Orden de Implementación Recomendado

```
F1 (Interfaz) → F3 (Minimap) → F4 (Fórmulas) → F2 (Joystick) → F5 (Cielo) → F6 (Post) → F7 (Exploración)
```

| # | Fase | Dificultad | Impacto | Dependencias |
|---|------|-----------|---------|-------------|
| 1 | Interfaz Renacida | Media | ⭐⭐⭐⭐⭐ | Ninguna |
| 2 | Minimap + Brújula | Media | ⭐⭐⭐⭐⭐ | F1 (parcial) |
| 3 | Universo de Fórmulas | Media-Alta | ⭐⭐⭐⭐ | Ninguna |
| 4 | Joystick + Control | Media | ⭐⭐⭐⭐ | Ninguna |
| 5 | Cielo Vivo | Media | ⭐⭐⭐⭐⭐ | F3 (parcial) |
| 6 | Post-procesado | Alta | ⭐⭐⭐⭐⭐ | F5 |
| 7 | Exploración Aumentada | Baja-Media | ⭐⭐⭐⭐ | F3 |

---

*Plan generado el 15 de Mayo 2026 — WORLDS Project*
