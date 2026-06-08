# WORLDS — Roadmap 10 Fases

> **Objetivo:** Seguridad → Optimización → Realismo Poligonal 100%
>
> Eliminar toda estética geométrica/Minecraft. Reemplazar primitivas (cajas, cilindros, esferas) por polígonos orgánicos reales con PBR, iluminación avanzada y post-procesado cinematográfico.

---

## Fase 1 — 🛡️ Seguridad del Sistema ✅
**Esfuerzo:** 6-8h | **Prioridad:** Crítica | **Completado**

| # | Tarea | Archivos |
|---|-------|----------|
| 1.1 | Sanitizar path traversal en `serve_asset` (bloquear `../`) | `server/src/main.rs` |
| 1.2 | CSP header + meta tag (Content Security Policy estricta) | `server/src/main.rs`, `client/index.html` |
| 1.3 | CORS restrictivo — lista blanca en vez de `allow_origin(Any)` | `server/src/main.rs` |
| 1.4 | Validar URL de mods — solo `https://`, límite de tamaño, sanitizar | `client/src/app.rs`, `client/src/engine/modding/mod.rs` |
| 1.5 | Validar WebSocket URL — solo `wss://` o localhost | `client/src/engine/mod.rs`, `client/three_bridge.js` |
| 1.6 | Rate limiting WebSocket + validar longitud de chat (max 500 chars) | `server/src/ws/mod.rs` |
| 1.7 | Origin validation en WebSocket upgrade | `server/src/ws/mod.rs` |
| 1.8 | SRI en CDNs + cabeceras de seguridad faltantes (HSTS, X-Frame-Options, etc.) | `client/index.html`, `client/service-worker.js`, `server/src/main.rs` |

---

## Fase 2 — ⚡ Optimización del Motor ✅
**Esfuerzo:** 8-10h | **Prioridad:** Alta | **Completado**

| # | Tarea | Archivos |
|---|-------|----------|
| 2.1 | Pool de Vecs reutilizables en generación de chunks (reducir allocaciones) | `client/src/engine/mod.rs`, `client/src/engine/chunk.rs` |
| 2.2 | Reducir llamadas bridge JS — batch tears de mesh uploads en un solo array tipado | `client/src/engine/bridge.rs`, `client/three_bridge.js` |
| 2.3 | LOD adaptativo basado en FPS target (ajustar render_distance dinámicamente) | `client/src/engine/mod.rs`, `client/src/engine/chunk.rs` |
| 2.4 | Memoizar cálculos de Gerstner waves (cache de posiciones/normales entre frames) | `client/src/engine/mod.rs`, `client/three_bridge.js` |
| 2.5 | Evitar clones de `WorldParams` en el loop principal (pasar referencias) | `client/src/engine/mod.rs` |
| 2.6 | Pool de vectores para partículas (reutilizar buffers) | `client/src/engine/particles.rs` |
| 2.7 | Debounce auto-save — solo persistir si hubo cambios reales | `client/src/engine/db.rs`, `client/src/engine/mod.rs` |

---

## Fase 3 — 🏔️ Subsuelo Marching Cubes ✅
**Esfuerzo:** 10-14h | **Depende de:** F2 | **Completado**

Eliminar el subsuelo de voxel-blocks. Reemplazar con isosuperficie extraída vía **marching cubes** sobre noise 3D.

| # | Tarea | Archivos |
|---|-------|----------|
| 3.1 | Implementar tabla de marching cubes (15 casos base + rotaciones) | `client/src/engine/chunk.rs` |
| 3.2 | Samplear noise 3D como campo de distancias firmadas (SDF) para cuevas, túneles, cavernas | `client/src/engine/chunk.rs`, `client/src/engine/terrain.rs` |
| 3.3 | Extraer isosuperficie con vértices interpolados (no en grid fijo) | `client/src/engine/chunk.rs` |
| 3.4 | Calcular normales desde gradiente del SDF (suavizado perfecto) | `client/src/engine/chunk.rs` |
| 3.5 | Reemplazar `emit_face()` + `push_face()` por la nueva malla | `client/src/engine/chunk.rs` |
| 3.6 | Adaptar LOD para marching cubes (menos resolución = menos cubos) | `client/src/engine/chunk.rs` |
| 3.7 | Eliminar `block_atlas_tile()`, `BLK_*` constants, `blocks` array (código muerto) | `client/src/engine/chunk.rs`, `client/src/engine/terrain.rs` |
| 3.8 | Underground features actuales (lava lakes, crystal geodes, fungus caverns, dungeons) → SDF booleano | `client/src/engine/chunk.rs` |

---

## Fase 4 — 🦌 Criaturas Orgánicas ✅
**Esfuerzo:** 8-12h | **Depende de:** F2 | **Completado**

~~Reemplazar `push_box()` por meshes poligonales reales para cada una de las 16 criaturas.~~

| # | Tarea | Archivos |
|---|-------|----------|
| ✅ 4.1 | Sistema de mallas orgánicas: `push_ellipsoid`, `push_cylinder` con dual mesh/positions | `client/src/engine/creatures.rs` |
| ✅ 4.2 | Criaturas terrestres: elipsoides + cilindros cónicos con patas animadas (trote/andadura) | `client/src/engine/creatures.rs` |
| ✅ 4.3 | Criaturas voladoras: alas como elipsoides delgados, aleteo sinusoidal | `client/src/engine/creatures.rs` |
| ✅ 4.4 | Criaturas acuáticas: cuerpo hidrodinámico, aletas, tentáculos como cilindros | `client/src/engine/creatures.rs` |
| ✅ 4.5 | Criaturas especiales: cristalino facetado, elemental pulsante, serpiente segmentada | `client/src/engine/creatures.rs` |
| ✅ 4.6 | Morfología variable por tipo (tamaño, proporciones fijas por especie) | `client/src/engine/creatures.rs` |
| ⬜ 4.7 | Animación por morph targets (deformación de vértices en CPU/GPU) — Pendiente | `client/src/engine/creatures.rs`, `client/three_bridge.js` |
| ✅ 4.8 | Eliminar `push_box()` y `push_box_positions()` de creatures | `client/src/engine/creatures.rs` |

---

## Fase 5 — 🏛️ Estructuras Poligonales ✅
**Esfuerzo:** 6-8h | **Depende de:** F2 | **Completado**

Reemplazar `push_box()` por arquitectura poligonal real. Los 13 tipos de estructura pasan a tener formas orgánicas.

| # | Tarea | Archivos |
|---|-------|----------|
| 5.1 | Arco poligonal real (curva de medio punto con dovelas) | `client/src/engine/structures.rs` |
| 5.2 | Cúpula geodésica con triangulación esférica (no cajas escalonadas) | `client/src/engine/structures.rs` |
| 5.3 | Columna con éntasis (curva griega) y capitel decorado | `client/src/engine/structures.rs` |
| 5.4 | Pirámide con caras poligonales suaves y bloques individuales | `client/src/engine/structures.rs` |
| 5.5 | Muralla con piedras irregulares individuales, no caja alargada | `client/src/engine/structures.rs` |
| 5.6 | Plaza con losas poligonales + fuente central modelada | `client/src/engine/structures.rs` |
| 5.7 | Puente con arco de medio punto real, pilares torneados, barandilla de forja | `client/src/engine/structures.rs` |
| 5.8 | Ruinas con piedras caídas poligonales individuales | `client/src/engine/structures.rs` |
| 5.9 | Dungeon Entrance con arco rebajado y escalones curvos | `client/src/engine/structures.rs` |
| 5.10 | Eliminar `push_box()` de structures (código muerto) | `client/src/engine/structures.rs` |

---

## Fase 6 — 🌿 Vegetación Orgánica ✅
**Esfuerzo:** 6-8h | **Depende de:** F2 | **Completado**

Reemplazar `push_cylinder()` + `push_sphere()` por polígonos vegetales reales.

| # | Tarea | Archivos |
|---|-------|----------|
| 6.1 | Troncos con forma anastomosada (nudos, curvatura, raíces en base) | `client/src/engine/vegetation.rs` |
| 6.2 | Ramas que bifurcan en Y con grosor decreciente (no cilindros rectos) | `client/src/engine/vegetation.rs` |
| 6.3 | Hojas como planos alfa con silueta de hoja real (múltiples orientaciones) | `client/src/engine/vegetation.rs`, `client/three_bridge.js` |
| 6.4 | Arbustos con volumen orgánico (esferoides deformados concatenados) | `client/src/engine/vegetation.rs` |
| 6.5 | Cactus con pliegues verticales y brazos curvos poligonales | `client/src/engine/vegetation.rs` |
| 6.6 | Flores con pétalos poligonales, tallo curvo, estambres | `client/src/engine/vegetation.rs` |
| 6.7 | Mushroom con sombrero y tallo de superficie orgánica (no cilindro+esfera) | `client/src/engine/vegetation.rs` |
| 6.8 | Grass como briznas individuales con curvatura (ya InstancedMesh, mejorar forma) | `client/src/engine/vegetation.rs`, `client/three_bridge.js` |
| 6.9 | Eliminar `push_cylinder()`, `push_sphere()`, `push_box()` de vegetation (código muerto) | `client/src/engine/vegetation.rs` |

---

## Fase 7 — 🧍 Personajes Humanoides ✅
**Esfuerzo:** 4-6h | **Depende de:** F2 | **Completado**

Reemplazar `cylinder_mesh()` + `uv_sphere()` + `box_mesh()` por mesh orgánico.

| # | Tarea | Archivos |
|---|-------|----------|
| 7.1 | Cuerpo con curvas anatómicas (tórax, caderas, hombros, cintura) | `client/src/engine/mod.rs` |
| 7.2 | Brazos/piernas con volumen variable (bíceps, antebrazo, muslo, pantorrilla) | `client/src/engine/mod.rs` |
| 7.3 | Cabeza con rasgos faciales (nariz, pómulos, mandíbula, arco orbital) | `client/src/engine/mod.rs` |
| 7.4 | Manos con dedos individuales (no manoplas cilíndricas) | `client/src/engine/mod.rs` |
| 7.5 | Kraken: tentáculos como splines poligonales con ventosas | `client/src/engine/mod.rs` |
| 7.6 | Robot: paneles biselados con uniones visibles (no caja) | `client/src/engine/mod.rs` |
| 7.7 | Animación walk/run con deformación de malla (músculos, estiramiento) | `client/src/engine/mod.rs` |
| 7.8 | Eliminar `cylinder_mesh()`, `box_mesh()`, `uv_sphere()` si no se usan más (código muerto) | `client/src/engine/mod.rs` |

---

## Fase 8 — 💎 Rocas, Minerales y Portales ✅
**Esfuerzo:** 3-4h | **Depende de:** F2 | **Completado**

Reemplazar `push_box()` por formas poligonales irregulares.

| # | Tarea | Archivos |
|---|-------|----------|
| 8.1 | Gemas con facetas poligonales tipo cristal real (icosfera deformada con planos de clivaje) | `client/src/engine/minerals.rs` |
| 8.2 | Rocas individuales con tessellation irregular (esferoides con ruido vértice) | `client/src/engine/minerals.rs` |
| 8.3 | Portal con anillo toroidal poligonal de alta resolución (no caja) | `client/src/engine/portals.rs` |
| 8.4 | Partículas de portal orgánicas (no puntos, sprites con forma) | `client/src/engine/portals.rs`, `client/three_bridge.js` |
| 8.5 | Eliminar `push_box()` de minerals y portals (código muerto) | `client/src/engine/minerals.rs`, `client/src/engine/portals.rs` |

---

## Fase 9 — 🎨 PBR + Materiales ✅
**Esfuerzo:** 6-8h | **Depende de:** F3–F8 | **Completado**

Texturas y materiales fotorrealistas para todas las mallas orgánicas.

| # | Tarea | Archivos |
|---|-------|----------|
| 9.1 | Normal mapping en terreno, criaturas, estructuras (reemplazar normales planas/vertex) | `client/three_bridge.js`, `client/src/engine/mod.rs` |
| 9.2 | Roughness/metalness maps por material (piedra, madera, metal, tierra, orgánico) | `client/three_bridge.js` |
| 9.3 | Detail texture con parallax occlusion mapping (mejorar resolución 64x64 → 256x256) | `client/three_bridge.js` |
| 9.4 | Atlas texture regenerado con aspecto realista (fotosourcing o procedural avanzado, no noise simple) | `client/three_bridge.js` |
| 9.5 | Texture arrays para terreno (mezcla de hasta 4 materiales por vértice con blending) | `client/three_bridge.js`, `client/src/engine/chunk.rs` |
| 9.6 | Colour grading LUT con perfiles cinematográficos por bioma (mejorar existing) | `client/three_bridge.js` |

---

## Fase 10 — ✨ Post-Procesado Final ✅
**Esfuerzo:** 8-10h | **Depende de:** F9 | **Completado**

Efectos visuales cinematográficos para realismo total.

| # | Tarea | Archivos |
|---|-------|----------|
| 10.1 | **Depth of Field** (BokehPass) — desenfoque de fondo suave | `client/three_bridge.js` |
| 10.2 | **PCSS** (Percentage Closer Soft Shadows) — sombras progresivas según distancia | `client/three_bridge.js` |
| 10.3 | **Contact Hardening** — sombras más nítidas cerca del contacto | `client/three_bridge.js` |
| 10.4 | **God Rays volumétricos** — rayos de luz desde antorchas y claros | `client/three_bridge.js` |
| 10.5 | **Baked Light Maps** — iluminación precalculada para chunks subterráneos | `client/src/engine/chunk.rs`, `client/three_bridge.js` |
| 10.6 | **Billboard Impostors** — cross-quads texturados para árboles lejanos | `client/three_bridge.js` |
| 10.7 | **Alpha-test shadow maps** — sombras correctas en vegetación con transparencia | `client/three_bridge.js` |
| 10.8 | **Anti-aliasing** — cambiar a FXAA o SMAA en post-process pipeline | `client/three_bridge.js` |

---

## Tabla Resumen

| Fase | Área | Esfuerzo | Depende de |
|------|------|----------|------------|
| ✅ | 🛡️ Seguridad | 6-8h | — |
| ✅ | ⚡ Optimización | 8-10h | — |
| ✅ | 🏔️ Marching Cubes subsuelo | 10-14h | F2 |
| ✅ | 🦌 Criaturas orgánicas | 8-12h | F2 |
| ✅ | 🏛️ Estructuras poligonales | 6-8h | F2 |
| ✅ | 🌿 Vegetación orgánica | 6-8h | F2 |
| ✅ | 🧍 Personajes humanoides | 4-6h | F2 |
| ✅ | 💎 Rocas/Minerales/Portales | 3-4h | F2 |
| ✅ | 🎨 PBR + Materiales | 6-8h | F3–F8 |
| ✅ | ✨ Post-Procesado final | 8-10h | F9 |
| | **Completado:** 1-10 | **~65-82h** | **100%** |

---

## Notas

- **Fases 1–10 completadas al 100%** ✅
- Única tarea pendiente: **4.7** (morph targets animación criaturas) — mejora opcional futura
- Cada fase incluye eliminar el código muerto de las primitivas reemplazadas

---

# WORLDS — Roadmap V2: Mundo Libre y Realista

> **Objetivo:** Exploración inmersiva total — poder adentrarse en cualquier espacio (agua, cuevas, montañas) con un mundo vivo, realista e interactivo.

---

## Fase 11 — 🌊 Exploración Submarina Inmersiva
**Esfuerzo:** 20-25h | **Prioridad:** Alta

| # | Tarea | Archivos |
|---|-------|----------|
| 11.1 | Sistema de buceo — barra de oxígeno, daño por ahogamiento, burbujas desde el jugador | `client/src/engine/mod.rs`, `client/three_bridge.js` |
| 11.2 | Cáusticas subacuáticas — proyección de ondas de luz en el fondo marino | `client/three_bridge.js` |
| 11.3 | Niebla subacuática — visibilidad reducida, color azul-verdoso, transición suave al salir | `client/three_bridge.js` |
| 11.4 | Flora acuática animada — algas, kelp, corales con sway por corriente | `client/src/engine/vegetation.rs`, `client/three_bridge.js` |
| 11.5 | Peces en cardúmenes — bancos de peces con IA de flocking | `client/src/engine/creatures.rs` |
| 11.6 | Efectos de profundidad — presión, oscuridad creciente, partículas de sedimentos | `client/src/engine/mod.rs`, `client/three_bridge.js` |
| 11.7 | Audio submarino — sonido ambiente muffled, burbujeo, latidos a presión | `client/src/engine/audio.rs` |

---

## Fase 12 — 🕯️ Iluminación y Exploración de Cuevas
**Esfuerzo:** 20-25h | **Prioridad:** Alta | **Depende de:** F11 (parcial)

| # | Tarea | Archivos |
|---|-------|----------|
| 12.1 | Sistema de linterna / antorcha — objeto craftable que emite luz 3D (PointLight) | `client/src/engine/inventory.rs`, `client/three_bridge.js`, `client/src/engine/mod.rs` |
| 12.2 | Linterna frontal en 1ª persona — luz que sigue la mirada del jugador | `client/three_bridge.js` |
| 12.3 | Cuevas más oscuras — eliminar el `min 0.25` de brightness, luz solo de fuentes | `client/src/engine/terrain.rs` |
| 12.4 | Estalactitas y estalagmitas — generación procedural en techos/suelos de cueva (SDF) | `client/src/engine/chunk.rs` |
| 12.5 | Ríos subterráneos — agua fluyendo dentro de cuevas | `client/src/engine/terrain.rs`, `client/src/engine/chunk.rs` |
| 12.6 | Lagos subterráneos — masas de agua en cámaras grandes | `client/src/engine/terrain.rs` |
| 12.7 | Cristales bioluminiscentes — emisivos que iluminan cuevas | `client/src/engine/minerals.rs` |
| 12.8 | Sonido de cueva — goteo, eco, reverberación | `client/src/engine/audio.rs` |

---

## Fase 13 — 🧗 Movimiento Vertical y Escalada
**Esfuerzo:** 15-20h | **Prioridad:** Media

| # | Tarea | Archivos |
|---|-------|----------|
| 13.1 | Escalada de paredes — agarrarse a superficies verticales (pendiente > 0.8) | `client/src/engine/mod.rs` |
| 13.2 | Salto de presa — agarrarse a bordes al saltar | `client/src/engine/mod.rs` |
| 13.3 | Deslizamiento por pendientes — física de deslizamiento en nieve/grava | `client/src/engine/mod.rs` |
| 13.4 | Natación contracorriente — nadar contra cascadas/ríos con esfuerzo | `client/src/engine/mod.rs` |
| 13.5 | Salto expandido — salto más alto con impulso (carrera + salto) | `client/src/engine/mod.rs` |

---

## Fase 14 — ⛏️ Interacción con el Terreno (Minería / Excavación)
**Esfuerzo:** 20-25h | **Prioridad:** Alta | **Depende de:** F12 (linterna)

| # | Tarea | Archivos |
|---|-------|----------|
| 14.1 | Sistema de minería — click izquierdo con pico destruye bloque apuntado | `client/src/engine/mod.rs`, `client/three_bridge.js` |
| 14.2 | Raycast para apuntado — detectar qué bloque/terreno mira el jugador | `client/three_bridge.js`, `client/src/engine/bridge.rs` |
| 14.3 | Partículas de rotura — escombros, polvo al minar | `client/src/engine/particles.rs` |
| 14.4 | Herramientas con durabilidad — picos se desgastan con uso | `client/src/engine/inventory.rs` |
| 14.5 | Colocación de bloques — construir estructuras en modo construcción (mejorar base existente) | `client/src/engine/mod.rs` |
| 14.6 | Sistema de crafteo expandido — antorchas, escaleras, cuerdas, botes | `client/src/engine/inventory.rs` |

---

## Fase 15 — 🌍 Mundo Vivo y Realista
**Esfuerzo:** 20-25h | **Prioridad:** Media

| # | Tarea | Archivos |
|---|-------|----------|
| 15.1 | Sistema de temperatura — frío en tundra/alta montaña, calor en desierto/lava | `client/src/engine/mod.rs` |
| 15.2 | Efectos de altitud — nieve en picos, viento fuerte, niebla de montaña | `client/src/engine/terrain.rs`, `client/three_bridge.js` |
| 15.3 | Erosión avanzada — barrancos, acantilados, deltas de ríos más realistas | `client/src/engine/erosion.rs` |
| 15.4 | Transiciones de bioma suaves — mezcla gradual entre zonas | `client/src/engine/terrain.rs` |
| 15.5 | Nubes volumétricas — nubes 3D que proyectan sombras | `client/three_bridge.js` |
| 15.6 | Niebla de valle — niebla baja en valles y cerca del agua al amanecer | `client/three_bridge.js` |

---

## Fase 16 — 📟 Interfaz y HUD de Exploración
**Esfuerzo:** 10-15h | **Prioridad:** Media | **Depende de:** F11, F12, F15

| # | Tarea | Archivos |
|---|-------|----------|
| 16.1 | Brújula — dirección cardinal en HUD | `client/src/app.rs`, `client/src/engine/mod.rs` |
| 16.2 | Altímetro — altitud actual sobre el nivel del mar | `client/src/app.rs` |
| 16.3 | Barra de oxígeno — al bucear | `client/src/app.rs` |
| 16.4 | Barra de temperatura — frío/calor | `client/src/app.rs` |
| 16.5 | Minimapa mejorado — con topografía, cuevas, waypoints | `client/src/engine/minimap.rs` |
| 16.6 | Indicador de herramientas — qué item está equipado | `client/src/app.rs` |

---

## Fase 17 — 🏆 Sistema de Progresión
**Esfuerzo:** 10-15h | **Prioridad:** Baja | **Depende de:** F14, F16

| # | Tarea | Archivos |
|---|-------|----------|
| 17.1 | Logros de exploración — descubrir cuevas, puntos profundos, cimas | `client/src/engine/achievements.rs` |
| 17.2 | Mapa del mundo — mapa desbloqueable que se dibuja al explorar | `client/src/engine/minimap.rs`, `client/three_bridge.js` |
| 17.3 | Puntos de viaje rápido — tótems/altares que permiten teletransporte | `client/src/engine/structures.rs` |
| 17.4 | Mejoras de equipo — pico mejorado, botas de escalada, aletas de buceo | `client/src/engine/inventory.rs` |

---

## Fase 18 — ⚡ Optimización para Mundo Abierto
**Esfuerzo:** 10-15h | **Prioridad:** Media | **Depende de:** F11–F15

| # | Tarea | Archivos |
|---|-------|----------|
| 18.1 | LOD adaptativo por bioma — menos detalle en áreas lejanas/subacuáticas | `client/src/engine/chunk.rs` |
| 18.2 | Culling de agua — no renderizar agua cuando el jugador está muy arriba | `client/three_bridge.js` |
| 18.3 | Occlusion culling para cuevas — no renderizar chunks subterráneos no visibles | `client/src/engine/chunk.rs` |
| 18.4 | Instancing de estalactitas — reducir draw calls en cuevas | `client/src/engine/chunk.rs`, `client/three_bridge.js` |

---

## Tabla Resumen V2

| Fase | Área | Esfuerzo | Depende de |
|------|------|----------|------------|
| ⬜ | 🌊 Exploración Submarina | 20-25h | — |
| ⬜ | 🕯️ Iluminación de Cuevas | 20-25h | F11 (parcial) |
| ⬜ | 🧗 Movimiento Vertical | 15-20h | — |
| ⬜ | ⛏️ Minería y Terreno | 20-25h | F12 |
| ⬜ | 🌍 Mundo Vivo | 20-25h | — |
| ⬜ | 📟 HUD de Exploración | 10-15h | F11, F12, F15 |
| ⬜ | 🏆 Progresión | 10-15h | F14, F16 |
| ⬜ | ⚡ Optimización | 10-15h | F11–F15 |
| | **Total V2** | **~125-165h** | |

## Prioridad de Implementación

1. **Fase 11** (Submarina) + **Fase 12** (Cuevas) — la base de la exploración
2. **Fase 14** (Minería) — interactuar con el mundo
3. **Fase 13** (Escalada) — acceso vertical
4. **Fase 15** (Mundo vivo) + **Fase 16** (HUD)
5. **Fase 17** (Progresión) + **Fase 18** (Optimización)
