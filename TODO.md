# WORLDS — Plan de Desarrollo Pendiente

Priorizado por impacto/dependencias. Cada fase lista archivos a modificar y tareas concretas.

---

## 1. ✅ F5 — Persistencia (Save/Load en IndexedDB)
**Impacto:** ⭐⭐⭐⭐ | **Depende de:** — | **Tamaño:** 2-3h **COMPLETADO**

- [x] Migrar `localStorage` a `IndexedDB` (almacenamiento más grande y estructurado)
- [x] Serializar: posición, waypoints, descubrimientos, seed, parámetros, día, inventario, logros, codex, bloques colocados
- [x] Ranuras de guardado (3 slots) con nombre, timestamp, screenshot thumbnail
- [x] Auto-save cada 30s + Save manual
- [x] Carga automática al iniciar (con diálogo "nuevo mundo" vs "continuar")
- [x] Load game desde el menú principal

**Archivos:** `app.rs`, `state/mod.rs`, `engine/mod.rs`, `index.html`

---

## 2. ✅ F7 — Terreno Voxel 3D (Cuevas reales)
**Impacto:** ⭐⭐⭐⭐⭐ | **Depende de:** F6 | **Tamaño:** 15-20h **COMPLETADO**

### 2a. Estructura de chunks 3D
- [x] Cambiar `ChunkData` de heightmap 2D a grid tridimensional (16×64×16)
- [x] Algoritmo de tallado 3D (noise 3D para cuevas, cavernas, túneles)
- [x] Face culling: solo renderizar caras visibles entre bloques sólidos y aire
- [x] Transición suave superficie↔subterráneo (blending de colores)

### 2b. Bloques individuales
- [x] Tipos de bloque: tierra, piedra, carbón, hierro, oro, diamante, lava, agua subterránea
- [x] Break animation (partículas al romper)
- [x] Drop de items al romper
- [x] Iluminación de antorcha (vertex lighting dinámico)

### 2c. Features subterráneas
- [x] Acuíferos subterráneos
- [x] Lagos de lava en profundidad (zona Magma)
- [x] Dungeons/Dungeon rooms generadas proceduralmente
- [x] Cristales gigantes (Crystal zone bajo tierra)

**Archivos:** `chunk.rs` (reforma mayor), `terrain.rs`, `engine/mod.rs`, `minerals.rs`, `three_bridge.js`, nuevo `engine/voxel.rs`

---

## 3. ✅ F8 — Ecosistemas Dinámicos
**Impacto:** ⭐⭐⭐⭐ | **Depende de:** F1 | **Tamaño:** 8-10h **COMPLETADO**

- [x] Sistema de estaciones (4 estaciones con duración configurable)
- [x] Cambio de color de follaje por estación
- [x] Crecimiento de árboles (semilla → brote → joven → adulto) con tick semanal
- [x] Flores y polinización (insectos voladores + frutos)
- [x] Frentes meteorológicos: sistemas de lluvia que se desplazan por el mapa
- [x] Animales de bioma (mariposas, peces, aves)

**Archivos:** `vegetation.rs`, `particles.rs`, `engine/mod.rs`, `terrain.rs`, `creatures.rs`, `app.rs`

---

## 4. ✅ F9 — Criaturas con IA
**Impacto:** ⭐⭐⭐⭐⭐ | **Depende de:** F7, F8 | **Tamaño:** 8-12h **COMPLETADO**

- [x] Pathfinding A* sobre grid del terreno
- [x] Comportamientos: deambular, huir, alimentarse, seguir
- [x] Spawn condicional: tipo de criatura según bioma, hora, clima
- [x] Animaciones (idle, walk, run, attack) con morph targets o rotación de partes
- [x] Criaturas de rescate ocultas en estructuras recompensa
- [x] Interacción: click para examinar, alimentar, domar
- [x] Criaturas montables (deer, bear)

**Archivos:** `creatures.rs` (reforma mayor), `engine/mod.rs`, `structures.rs`, `three_bridge.js`, `controls.rs`

---

## 5. ✅ F10 — Audio 3D Inmersivo
**Impacto:** ⭐⭐⭐⭐ | **Depende de:** — | **Tamaño:** 5-8h **COMPLETADO**

- [x] `PannerNode` para audio posicional 3D (criaturas, portales, listener)
- [x] Crossfade entre paisajes sonoros de biomas
- [x] Sistema musical dinámico (bass + pad por bioma, modulado por altura/velocidad/día)
- [x] Pasos con sonido según superficie (pasto, piedra, agua, madera)
- [x] Eco en cuevas con `ConvolverNode` + impulso procedural
- [x] Sonido de lluvia graduado por intensidad
- [x] Audio de portales (zumbido, distorsión al atravesar, now spatial)

**Archivos:** `audio.rs` (reforma mayor), `engine/mod.rs`

---

## 6. ✅ F11 — Portales (mejora)
**Impacto:** ⭐⭐⭐⭐ | **Depende de:** F5 | **Tamaño:** 3-5h **COMPLETADO**

- [x] Shader de distorsión anular en el plano del portal (ShaderMaterial con colores cíclicos y pulsación)
- [x] Partículas alrededor del portal (30 partículas flotando, rotación y bobleo)
- [x] Al atravesar: animación de fade-out/fade-in (0.35s → negro → teleport → 0.35s → claro)
- [x] Historial de mundos visitados (portal hub en UI con seeds visitados)
- [x] Múltiples portales por mundo con destinos diferentes (info en HUD: seed destino)

**Archivos:** `portals.rs`, `three_bridge.js`, `engine/mod.rs`, `app.rs`, `bridge.rs`, `state/mod.rs`

---

## 7. ✅ F13 — Hidrología
**Impacto:** ⭐⭐⭐⭐ | **Depende de:** F7, F10 | **Tamaño:** 8-10h **COMPLETADO**

- [x] Ríos: cauces tallados con noise sinusoidal, agua superficial en canales
- [ ] Puentes sobre ríos (depende de F17 Arquitectura)
- [x] Oleaje en costa (vertex displacement en shader de agua)
- [x] Flora acuática: algas, corales, kelp (mesh instances en zona acuática)
- [x] Espuma en costa (foam)
- [x] Cascadas con partículas
- [x] Burbujas bajo el agua (partículas ascendentes)
- [x] Sonido de cascadas con audio espacial (requiere F10)

**Archivos:** `terrain.rs`, `particles.rs`, `waterfall.rs`, `audio.rs`, `engine/mod.rs`

---

## 8. ✅ F17 — Arquitectura
**Impacto:** ⭐⭐⭐⭐ | **Depende de:** — | **Tamaño:** 4-6h **COMPLETADO**

- [x] Plazas: áreas despejadas rodeadas de estructuras
- [x] Puentes sobre agua entre caminos
- [x] Murallas/perímetros defensivos alrededor de núcleos
- [x] Variedad arquitectónica por bioma (estilo, material, color)
- [x] Dungeons subterráneos debajo de estructuras grandes (Plaza, Pyramid, Tower, Dome)

**Archivos:** `structures.rs`, `chunk.rs`

---

## 9. ✅ F19 — Modding API
**Impacto:** ⭐⭐⭐⭐⭐ | **Depende de:** — | **Tamaño:** 8-10h **COMPLETADO**

- [x] Definiciones de biomas en JSON (cargar desde URL vía `?mod=`)
- [x] Plugins de fórmulas matemáticas (eval en runtime vía expresión: sin, cos, sqrt, clamp, etc.)
- [x] Blueprints de estructuras en formato declarativo (bloques con posición, tamaño, color)
- [x] Paletas de color personalizadas (JSON, sobreescriben colores de bloques por nombre)
- [x] Compartir mods via URL: `?mod=https://.../biome.json`
- [x] Biome `Custom(u32)` para biomas completamente nuevos (con altura, vegetación, estructuras)
- [x] Overrides para biomas existentes (colores, densidad vegetación/estructuras, gradientes)

**Archivos:** nuevo `engine/modding/mod.rs`, `engine/modding/formula.rs`, `engine/modding/biome.rs`, `engine/modding/blueprint.rs`, `engine/terrain.rs`, `engine/structures.rs`, `engine/vegetation.rs`, `engine/mod.rs`, `app.rs`, `audio.rs`, `particles.rs`, `Cargo.toml`

---

## 10. ✅ F20 — Optimización & Pulido
**Impacto:** ⭐⭐⭐⭐⭐ | **Depende de:** Todas | **Tamaño:** continuo **COMPLETADO**

- [x] LOD: chunks lejanos con menos vértices (3 niveles, sample step 1/2/4)
- [x] LOD: render_distance aumentado de 2→4 (9×9 chunks) con LOD progresivo
- [x] Frustum culling (GPU: no enviar chunks fuera de la vista) — habilitado + bounding spheres
- [ ] Web Workers: generación de chunks en worker separado (→ F19)
- [x] PWA: manifest.json + icon.svg + favicon + service worker
- [x] Responsive: UI adaptada a móvil (botones más pequeños en <sm, columnas ocultas)
- [x] i18n: ES/EN/FR/DE/JA en JSON de traducciones + módulo de carga
- [x] URL Sharing: `?seed=&zone=&scale=&amplitude=&water=&octaves=&canyons=&mutation=&speed=&fly=&hue=&saturation=&char=&particles=`
- [x] Accesibilidad: aria-labels en botones, focus ring, role="application" en canvas

**Archivos:** múltiples

---

## 11. 🚀 F19 — Web Workers & Optimización Profunda
**Impacto:** ⭐⭐⭐⭐⭐ | **Depende de:** — | **Tamaño:** 8-12h

- [ ] Worker dedicado para generación de chunks (WASM en worker)
- [ ] Cola de prioridad: chunks cercanos primero
- [ ] Streaming: chunks generados en segundo plano
- [ ] Profiling: panel FPS/draw calls/memoria en HUD

---

## 12. 🚀 F20 — Mejoras Mobile
**Impacto:** ⭐⭐⭐⭐ | **Depende de:** — | **Tamaño:** 4-6h

- [ ] Joystick táctil redimensionable
- [ ] Gestos: tap para interactuar, swipe para cámara
- [ ] HUD adaptado al pulgar
- [ ] Fullscreen automático en móvil
- [ ] Degradado automático de calidad en móvil

---

## 13. 🚀 F21 — Sistema de Bosses
**Impacto:** ⭐⭐⭐⭐⭐ | **Depende de:** F7, F9 | **Tamaño:** 6-10h

- [ ] 3 bosses: Golem de piedra, Kraken de lava, Guardián de cristal
- [ ] Salas de boss en dungeons
- [ ] IA con fases de ataque y patrones
- [ ] Barra de vida en HUD
- [ ] Botín exclusivo

---

## 14. 🚀 F22 — Misiones y Narrativa Procedural
**Impacto:** ⭐⭐⭐⭐⭐ | **Depende de:** F15, F21 | **Tamaño:** 8-12h

- [ ] Sistema de misiones procedurales
- [ ] NPCs con diálogo en estructuras
- [ ] Marcadores en mapa
- [ ] Recompensas: paletas, cosméticos, zonas secretas
- [ ] Secuencia de historia de 5 misiones por mundo

---

---

## 15. 🌿 F-Orgánica — Transformación a Estética Orgánica/Realista
**Impacto:** ⭐⭐⭐⭐ | **Depende de:** — | **Tamaño:** continuo

Eliminar toda estética geométrica/Minecraft. Transformar a orgánico/realista.

### Fase 1 — Eliminar sistema de bloques ✅
- [x] Eliminar `placed_blocks`, `mined_blocks`, `block_inventory` del GameState
- [x] Eliminar build mode, hotbar (teclas B, 1-9), break particles
- [x] Eliminar `collides_with_blocks()`, `raycast_block()`, block serialization
- [x] Compila limpio (0 warnings)

### Fase 2 — Vegetación orgánica ✅
- [x] `push_cylinder()` y `push_sphere()` en `vegetation.rs`
- [x] Tree: tronco cilíndrico + copa esférica
- [x] Bush: esfera
- [x] Cactus: cilindros (tronco + brazo)
- [x] DeadTree: tronco cilíndrico
- [x] Mushroom: tallo cilíndrico + sombrero esférico
- [x] Compila limpio (0 warnings)

### Fase 3 — Personaje orgánico ✅
**Archivos:** `engine/mod.rs`
- [x] Añadidas `cylinder_mesh()` y `cylinder_pivot_top()` (cilindro con tapas, con/sin pivote superior)
- [x] Body: reemplazado de `box_mesh` a `cylinder_mesh` (radio = (bw+bd)*0.25, altura = bh, 8 segmentos)
- [x] Arms/Legs: reemplazados de `box_pivot_top` a `cylinder_pivot_top` (radio = (aw+ad)*0.25, altura = ah, 8 segmentos)
- [x] Head: ya era `uv_sphere` para la mayoría de presets (se mantiene)
- [x] Robot head: mantiene `box_mesh` (intencional, estética robot)
- [x] Kraken tentáculos: cambiados a cilindros delgados
- [x] Eliminada `box_pivot_top` (código muerto)
- [x] Compila limpio (0 warnings)

### Fase 4 — Subsuelo suavizado ✅
**Archivos:** `chunk.rs`
- [x] Post-process de normales: mapear posiciones de vértices → normales acumuladas vía HashMap
- [x] Normalizar normales promediadas para cada vértice del subsuelo
- [x] Afecta solo caras planas (flat-shaded), la superficie (smooth) queda intacta
- [x] Compila limpio (0 warnings)

### Fase 5 — Controles fauna ✅
**Archivos:** `controls.rs`, `engine/mod.rs`
- [x] Click derecho (`MASK_RCLICK`) ya interactúa con fauna: alimenta con fruta o examina (nombre + bioma)
- [x] Click izquierdo (`MASK_LCLICK`) no tiene acción (block placement eliminado en Fase 1)
- [x] Ya implementado desde Fase 1 — solo verificar

## Resumen de Prioridades

| Prio | Fase | Esfuerzo | Por qué ahora |
|------|------|----------|---------------|
| **1** | ✅ F5 Persistencia | Hecho | IndexedDB auto-save |
| **2** | ✅ F7 Voxel 3D | Hecho | 32 capas, cuevas, dungeons |
| **3** | ✅ F8 Ecosistemas | Hecho | Estaciones, crecimiento, frutos, clima |
| **1** | ✅ F9 Criaturas IA | Hecho | Animaciones, rescate, montura |
| **2** | ✅ F10 Audio 3D | Hecho | PannerNode, reverb, música dinámica |
| **3** | ✅ F13 Hidrología | Hecho | Ríos, burbujas, sonido cascadas |
| **4** | ✅ F11 Portales | Hecho | Shader distorsión, fade, hub, partículas |
| **5** | ✅ F17 Arq. | Hecho | Plazas, puentes, murallas, dungeons |
| **6** | ✅ F19 Modding | Hecho | Biomas JSON, fórmulas, blueprints, paletas |
| **7** | ✅ F20 Optimizar | Hecho | LOD, frustum, PWA, i18n, responsive |
| **8** | 🚀 F19 Web Workers | 8-12h | Rendimiento: chunks en worker |
| **9** | 🚀 F20 Mobile | 4-6h | Experiencia táctil pulida |
| **10** | 🚀 F21 Bosses | 6-10h | Jefes en mazmorras |
| **11** | 🚀 F22 Misiones | 8-12h | Narrativa procedural |
