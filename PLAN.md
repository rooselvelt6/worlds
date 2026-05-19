# 🌍 WORLDS — Super Plan de 20 Fases

## Estado Actual (Completado ~85% del plan anterior)
- ✅ 27 fórmulas matemáticas (builtins.rs)
- ✅ 15 biomas (terrain.rs)
- ✅ Glassmorphism UI + 5 tabs (app.rs)
- ✅ Joystick táctil (joystick.rs)
- ✅ Minimapa circular + brújula + altímetro (minimap.rs)
- ✅ Partículas ambientales por bioma (particles.rs)
- ✅ Ciclo día/noche + niebla adaptativa
- ✅ Post-procesado UnrealBloomPass + viñeta
- ✅ Waypoints + modo observador + screenshots
- ✅ Minería (8 minerales con emisión)
- ✅ Estructuras ocultas (10 tipos)
- ✅ Audio 100% sintetizado por bioma
- ✅ Sistema de descubrimiento de biomas
- ✅ Exportación OBJ/STL

---

## Visión General

Evolucionar WORLDS de un explorador de mundos procedurales a un **ecosistema interactivo completo** con multijugador, construcción, criaturas, VR, y una plataforma de modding.

---

## FASE 1: Blending de Fórmulas & Mutación Procedural ✅
**Objetivo:** Combinar fórmulas y que cada chunk sea único.

- [x] Slider `Mezcla` (0-1 entre fórmula actual y secundaria)
- [x] Selector de `Fórmula B` (dropdown en tab Fórmula)
- [x] Mutación por chunk: variación ligera de scale/amplitude según `seed + cx + cz`
- [x] Slider `Mutación` en tab Avanzado
- [x] Color blend: colores interpolados entre ambas fórmulas

**Archivos:** `state/mod.rs`, `terrain.rs`, `chunk.rs`, `app.rs`

---

## FASE 2: Gamepad API Completo ✅
**Objetivo:** Soporte total para mandos USB/bluetooth.

- [x] Ejes analógicos → movimiento/rotación (left stick move, right stick camera)
- [x] Botones A/B/X/Y → saltar/volar/screenshot/interactuar
- [x] LB/RB → rotar cámara (Q/E)
- [x] D-pad → movimiento direccional
- [x] Indicador visual de gamepad conectado
- [x] Polling cada frame vía `navigator.getGamepads()`

**Archivos:** `gamepad.rs` (nuevo), `engine/mod.rs`, `app.rs`

---

## FASE 3: Post-Procesado Cinematográfico (DoF + LUT) ✅
**Objetivo:** Efectos de profundidad de campo y grading por bioma.

- [x] Depth of Field (desenfoque lejano suave con ShaderPass)
- [x] LUT (Look-Up Table) por bioma: paletas de color únicas por preset
- [x] Mejora de bloom existente con máscaras por zona (intensidad por bioma)
- [x] Efecto "heat haze" en zonas Volcanic/Magma
- [x] Importación de addons Three.js vía ES modules dinámicos (jsm)
- [x] LUT generado en canvas con contraste/vibrancia/calor por bioma
- [x] DoF con blur gaussiano dependiente de profundidad
- [x] Heat haze con distorsión sinusoidal de UV

**Archivos:** `three_bridge.js`, `index.html`, `bridge.rs`, `audio.rs`

---

## FASE 4: Servidor Multijugador (WebSockets) ✅
**Objetivo:** Compartir mundos en tiempo real.

- [x] Servidor WebSocket con `axum::extract::ws` en el backend Axum
- [x] Sincronización de posición de jugadores (broadcast por sala)
- [x] Cada jugador ve a otros como figuras geométricas básicas (cápsula + esfera)
- [x] Sistema de salas (misma seed = mismo mundo, rooms keyed by seed)
- [x] Latencia compensada con interpolación lineal (factor 0.15)
- [x] Conexión WebSocket desde WASM via bridge → JS nativo
- [x] Renderizado de jugadores remotos con cuerpo + cabeza + anillo de suelo
- [x] Limpieza automática de salas vacías

**Archivos:** `server/src/ws/mod.rs`, `server/src/main.rs`, `three_bridge.js`, `bridge.rs`, `engine/mod.rs`

---

## FASE 5: Persistencia (Save/Load en IndexedDB)
**Objetivo:** Guardar y restaurar estado del mundo localmente.

- [ ] Serialización de: posición, waypoints, descubrimientos, seed, parámetros
- [ ] Almacenamiento en IndexedDB vía `web-sys`
- [ ] Carga automática al iniciar (con opción de "nuevo mundo")
- [ ] Ranuras de guardado múltiples (3 slots)

**Archivos:** `app.rs`, `state/mod.rs`, `bridge.rs`, `three_bridge.js`

---

## FASE 6: Minería & Construcción
**Objetivo:** Interactuar con el mundo: minar minerales y construir.

- [ ] Click izquierdo → minar bloque (voxel si está cerca)
- [ ] Inventario básico (minerales recolectados)
- [ ] Click derecho → colocar bloque seleccionado
- [ ] Sistema de "build mode" toggle
- [ ] Crafting básico (fusionar minerales)

**Archivos:** `controls.rs`, `engine/mod.rs`, `bridge.rs`, `three_bridge.js`, nuevo `engine/inventory.rs`

---

## FASE 7: Terreno Voxel 3D (Subterráneo Mejorado)
**Objetivo:** Cuevas y cavernas reales con tallado 3D.

- [ ] Sistema de chunks 3D (altura × ancho × profundidad)
- [ ] Bloques individuales con caras visibles (face culling)
- [ ] Iluminación de antorcha para cuevas
- [ ] Transición suave superficie ↔ subterráneo
- [ ] Acuíferos subterráneos y lagos de lava

**Archivos:** `chunk.rs`, `terrain.rs`, `engine/mod.rs`, `three_bridge.js`

---

## FASE 8: Ecosistemas Dinámicos
**Objetivo:** Flora y clima que evolucionan con el tiempo.

- [ ] Crecimiento de árboles (etapas: semilla → brote → adulto)
- [ ] Frentes meteorológicos que se desplazan (lluvia → sol → tormenta)
- [ ] Ciclo de estaciones (4 estaciones, cambio de colores de follaje)
- [ ] Polinización: flores atraen insectos → árboles frutales

**Archivos:** `vegetation.rs`, `particles.rs`, `audio.rs`, `three_bridge.js`, `engine/mod.rs`

---

## FASE 9: Criaturas & NPCs
**Objetivo:** Vida animal y encuentros.

- [ ] Criaturas por bioma (ciervos, serpientes, luciérnagas, etc.)
- [ ] Pathfinding simple (grid A* sobre terreno)
- [ ] Comportamiento: deambular, huir del jugador, alimentarse
- [ ] Criaturas de rescate ocultas en estructuras (recompensa)
- [ ] Animación básica con morph targets

**Archivos:** nuevo `engine/creatures.rs`, `engine/mod.rs`, `three_bridge.js`

---

## FASE 10: Revolución de Audio 3D
**Objetivo:** Audio inmersivo espacial.

- [ ] Web Audio API con `PannerNode` para audio 3D posicional
- [ ] Transiciones suaves entre paisajes sonoros de biomas (crossfade)
- [ ] Sistema musical dinámico (responde a altura, velocidad, hora)
- [ ] Pasos con distinto sonido según superficie
- [ ] Eco en cuevas con `ConvolverNode`

**Archivos:** `audio.rs`, `three_bridge.js`

---

## FASE 11: Sistema de Portales
**Objetivo:** Viajar entre mundos/semillas.

- [ ] Portales visuales (shader de distorsión anular)
- [ ] Al atravesar: cambia la seed, generando un nuevo mundo
- [ ] Historial de mundos visitados (portal hub)
- [ ] Efecto de transición con distorsión + fade

**Archivos:** `engine/mod.rs`, `three_bridge.js`, `app.rs`, `bridge.rs`

---

## FASE 12: Logros & Sistema de Progresión
**Objetivo:** Metas de exploración y recompensas.

- [ ] Logros: "Visita todos los biomas", "Descubre 10 estructuras", "Camina 10km"
- [ ] Recompensas: paletas de color, fórmulas secretas, modos visuales
- [ ] Notificaciones elegantes con glifo del logro
- [ ] Seguimiento persistente en IndexedDB

**Archivos:** nuevo `engine/achievements.rs`, `app.rs`, `state/mod.rs`

---

## FASE 13: Hidrología Avanzada
**Objetivo:** Agua dinámica con ríos, cascadas, oleaje.

- [ ] Ríos que fluyen cuesta abajo desde altura > water_level
- [ ] Cascadas con partículas de espuma y sonido 3D posicional
- [ ] Oleaje en la costa (vertex displacement en shader de agua)
- [ ] Flora acuática subacuática (algas, corales)
- [ ] Burbujas bajo el agua

**Archivos:** `terrain.rs`, `chunk.rs`, `three_bridge.js`, `particles.rs`, `audio.rs`

---

## FASE 14: Poderes Climáticos
**Objetivo:** El jugador puede influir en el clima.

- [ ] HUD de "poderes": invocar lluvia, despejar niebla, acelerar el día, llamar un rayo
- [ ] Enfriamiento por uso (cooldown con animación de anillo)
- [ ] Sinergia con biomas: lluvia en desierto → florecimiento temporal
- [ ] Efectos visuales y de audio para cada poder

**Archivos:** `app.rs`, `audio.rs`, `engine/mod.rs`, `three_bridge.js`

---

## FASE 15: Características Sociales
**Objetivo:** Conectar jugadores (requiere F4).

- [ ] Código de amigo para agregar contactos
- [ ] Visitar el mundo de otro jugador (teletransporte)
- [ ] Chat básico con burbujas sobre los jugadores
- [ ] Sistema de "favoritos": marcar mundos de amigos
- [ ] Co-op: chunks compartidos en tiempo real

**Archivos:** `engine/mod.rs`, `app.rs`, `three_bridge.js`, `server/src/ws/`

---

## FASE 16: Codex / Bestiario
**Objetivo:** Enciclopedia viva del mundo.

- [ ] Codex visual con pestañas: Biomas, Fórmulas, Estructuras, Minerales, Criaturas
- [ ] Cada entrada se desbloquea al descubrir/ver por primera vez
- [ ] Ilustraciones generadas proceduralmente (captura de cámara)
- [ ] Estadísticas de exploración

**Archivos:** nuevo `engine/codex.rs`, `app.rs`, `state/mod.rs`

---

## FASE 17: Arquitectura & Civilización Procedural
**Objetivo:** Ciudades en ruinas, caminos, puentes.

- [ ] Algoritmo de caminos que conectan estructuras cercanas
- [ ] Plazas, puentes sobre agua, murallas alrededor de núcleos
- [ ] Variedad arquitectónica por bioma (estilo, material, color)
- [ ] Dungeons subterráneos debajo de estructuras grandes

**Archivos:** `structures.rs`, `terrain.rs`, `three_bridge.js`, nuevo `engine/architecture.rs`

---

## FASE 18: Realidad Virtual (WebXR)
**Objetivo:** Exploración inmersiva en VR.

- [ ] Sesión WebXR con `THREE.WebXRManager`
- [ ] Movimiento por teleportación + joystick analógico
- [ ] Interacción con manos (recoger minerales, saludar criaturas)
- [ ] Escalado de UI para cascos VR (interfaz flotante)
- [ ] 72fps optimizado para VR

**Archivos:** `three_bridge.js`, `controls.rs`, `camera.rs`, `app.rs`

---

## FASE 19: Modding API & Contenido Generado por el Usuario
**Objetivo:** La comunidad puede extender el juego.

- [ ] Definiciones de biomas en TOML/JSON
- [ ] Plugins de fórmulas matemáticas (expresiones evaluadas en runtime)
- [ ] Blueprints de estructuras en formato declarativo
- [ ] Paletas de color personalizadas (subidas como JSON)
- [ ] Compartir mods via URL: `worlds.app/?mod=https://.../biome.toml`

**Archivos:** nuevo `modding/`, `engine/mod.rs`, `app.rs`, `three_bridge.js`, `Cargo.toml`

---

## FASE 20: Optimización & Pulido General
**Objetivo:** Rendimiento, accesibilidad y distribución.

- [ ] LOD (Level of Detail): chunks lejanos con menos vértices
- [ ] Frustum culling: no renderizar chunks fuera de la vista
- [ ] Web Workers: generación de chunks en worker separado
- [ ] Mobile PWA: manifest.json, service worker, fullscreen táctil
- [ ] Accesibilidad: soporte de lector de pantalla, contraste, tamaño de fuente
- [ ] Internacionalización (i18n): JSON de traducciones (EN/ES/FR/DE/JA)
- [ ] URL Sharing: `worlds.app/?seed=12345&formula=Voronoi`

**Archivos:** múltiples — optimizaciones en engine, three_bridge, index.html, server

---

## Resumen de Impacto y Dependencias

| # | Fase | Dif. | Impacto | Dep. de |
|---|------|------|---------|---------|
| 1 | Blending & Mutación ✅ | Media | ⭐⭐⭐⭐ | — |
| 2 | Gamepad API ✅ | Media | ⭐⭐⭐ | — |
| 3 | DoF + LUT ✅ | Alta | ⭐⭐⭐⭐⭐ | — |
| 4 | Multijugador WS ✅ | Muy Alta | ⭐⭐⭐⭐⭐ | — |
| 5 | Persistencia | Media | ⭐⭐⭐⭐ | — |
| 6 | Minería/Construcción | Alta | ⭐⭐⭐⭐⭐ | F5 |
| 7 | Voxel 3D | Muy Alta | ⭐⭐⭐⭐⭐ | F6 |
| 8 | Ecosistemas | Alta | ⭐⭐⭐⭐ | F1 |
| 9 | Criaturas | Alta | ⭐⭐⭐⭐⭐ | F7, F8 |
| 10 | Audio 3D | Alta | ⭐⭐⭐⭐ | — |
| 11 | Portales | Media | ⭐⭐⭐⭐ | F5 |
| 12 | Logros | Baja | ⭐⭐⭐ | F5 |
| 13 | Hidrología | Alta | ⭐⭐⭐⭐ | F7 |
| 14 | Poderes Climáticos | Media | ⭐⭐⭐ | — |
| 15 | Social | Alta | ⭐⭐⭐⭐⭐ | F4 |
| 16 | Codex | Media | ⭐⭐⭐ | F9 |
| 17 | Arquitectura | Alta | ⭐⭐⭐⭐ | — |
| 18 | VR (WebXR) | Muy Alta | ⭐⭐⭐⭐⭐ | F3 |
| 19 | Modding API | Muy Alta | ⭐⭐⭐⭐⭐ | F20 |
| 20 | Optimización/Pulido | Media | ⭐⭐⭐⭐⭐ | Todas |

---

*Plan generado el 19 de Mayo 2026 — WORLDS Project*
