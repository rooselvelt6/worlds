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

## 4. F9 — Criaturas con IA
**Impacto:** ⭐⭐⭐⭐⭐ | **Depende de:** F7, F8 | **Tamaño:** 8-12h (parcial)

- [x] Pathfinding A* sobre grid del terreno
- [x] Comportamientos: deambular, huir, alimentarse, seguir
- [x] Spawn condicional: tipo de criatura según bioma, hora, clima
- [ ] Animaciones (idle, walk, run, attack) con morph targets o rotación de partes
- [ ] Criaturas de rescate ocultas en estructuras con recompensa
- [x] Interacción: click para examinar, alimentar, domar
- [ ] Criaturas montables (caballo, dragón pequeño)

**Archivos:** `creatures.rs` (reforma mayor), `engine/mod.rs`, `structures.rs`, `three_bridge.js`, `controls.rs`

---

## 5. F10 — Audio 3D Inmersivo
**Impacto:** ⭐⭐⭐⭐ | **Depende de:** — | **Tamaño:** 5-8h

- [ ] `PannerNode` para audio posicional 3D (cascadas, criaturas, pasos)
- [ ] Crossfade entre paisajes sonoros de biomas
- [ ] Sistema musical dinámico (responde a altura, velocidad, hora del día)
- [ ] Pasos con sonido según superficie (pasto, piedra, agua, madera)
- [ ] Eco en cuevas con `ConvolverNode`
- [ ] Sonido de lluvia graduado por intensidad
- [ ] Audio de portales (zumbido, distorsión al atravesar)

**Archivos:** `audio.rs` (reforma mayor), `three_bridge.js`, `engine/mod.rs`

---

## 6. F11 — Portales (mejora)
**Impacto:** ⭐⭐⭐⭐ | **Depende de:** F5 | **Tamaño:** 3-5h

- [ ] Shader de distorsión anular en el plano del portal (efecto visual)
- [ ] Partículas alrededor del portal
- [ ] Al atravesar: animación de fade-out/fade-in
- [ ] Historial de mundos visitados (portal hub en UI)
- [ ] Múltiples portales por mundo con destinos diferentes

**Archivos:** `portals.rs`, `three_bridge.js`, `engine/mod.rs`, `app.rs`

---

## 7. F13 — Hidrología (completar)
**Impacto:** ⭐⭐⭐⭐ | **Depende de:** F7 | **Tamaño:** 6-8h (parcial)

- [ ] Ríos: detectar cauces naturales desde altura hasta water_level (erosion simulation)
- [ ] Puentes sobre ríos (conectar caminos F17)
- [x] Oleaje en costa (vertex displacement en shader de agua)
- [x] Flora acuática: algas, corales, kelp (mesh instances en zona acuática)
- [x] Espuma en costa (foam)
- [x] Cascadas con partículas
- [ ] Burbujas bajo el agua (partículas ascendentes)
- [ ] Sonido de cascadas (requiere F10)

**Archivos:** `terrain.rs`, `three_bridge.js`, `particles.rs`, `waterfall.rs`, `structures.rs`, `engine/mod.rs`

---

## 8. F15 — Social & Multijugador (completar)
**Impacto:** ⭐⭐⭐⭐⭐ | **Depende de:** F4 | **Tamaño:** 5-8h

- [ ] Chat con burbujas sobre jugadores
- [ ] Comandos de sala: /invite, /whisper, /tp
- [ ] Indicador de amigos online
- [ ] Co-op: ambos jugadores ven cambios (bloques, minería)
- [ ] Sincronización de clima y hora del día
- [ ] Lobby con lista de mundos públicos

**Archivos:** `server/src/ws/mod.rs`, `server/src/main.rs`, `engine/mod.rs`, `app.rs`, `three_bridge.js`

---

## 9. F17 — Arquitectura (completar)
**Impacto:** ⭐⭐⭐⭐ | **Depende de:** — | **Tamaño:** 4-6h

- [ ] Plazas: áreas despejadas rodeadas de estructuras
- [ ] Puentes sobre agua entre caminos
- [ ] Murallas/perímetros defensivos alrededor de núcleos
- [ ] Variedad arquitectónica por bioma (estilo, material, color mejorado)
- [ ] Dungeons subterráneos debajo de estructuras grandes (requiere F7)

**Archivos:** `structures.rs` (reforma mayor), `terrain.rs`, `three_bridge.js`, nuevo `engine/architecture.rs`

---

## 10. F18 — Realidad Virtual (WebXR)
**Impacto:** ⭐⭐⭐⭐⭐ | **Depende de:** F3 | **Tamaño:** 10-15h

- [ ] Sesión WebXR inmersiva con `THREE.WebXRManager`
- [ ] Movimiento por teleportación + joystick
- [ ] Interacción con manos (recoger, construir, saludar)
- [ ] UI flotante adaptada a VR
- [ ] Optimización 72fps

**Archivos:** `three_bridge.js` (reforma mayor), `controls.rs`, `camera.rs`, `app.rs`, `index.html`

---

## 11. F19 — Modding API
**Impacto:** ⭐⭐⭐⭐⭐ | **Depende de:** F20 | **Tamaño:** 8-10h

- [ ] Definiciones de biomas en TOML/JSON (cargar desde URL o archivo)
- [ ] Plugins de fórmulas matemáticas (eval en runtime vía expresión)
- [ ] Blueprints de estructuras en formato declarativo
- [ ] Paletas de color personalizadas (JSON)
- [ ] Compartir mods via URL: `?mod=https://.../biome.toml`

**Archivos:** nuevo `modding/`, `engine/mod.rs`, `app.rs`, `three_bridge.js`, `Cargo.toml`

---

## 12. F20 — Optimización & Pulido
**Impacto:** ⭐⭐⭐⭐⭐ | **Depende de:** Todas | **Tamaño:** continuo

- [ ] LOD: chunks lejanos con menos vértices
- [ ] Frustum culling (GPU: no enviar chunks fuera de la vista)
- [ ] Web Workers: generación de chunks en worker separado
- [ ] PWA: manifest.json, service worker, instalable
- [ ] Responsive: UI adaptada a móvil
- [ ] i18n: ES/EN/FR/DE/JA en JSON de traducciones
- [ ] URL Sharing: `?seed=12345&formula=Voronoi`
- [ ] Accesibilidad: contraste, foco, lector de pantalla

**Archivos:** múltiples

---

## Resumen de Prioridades

| Prio | Fase | Esfuerzo | Por qué ahora |
|------|------|----------|---------------|
| **1** | ✅ F5 Persistencia | Hecho | IndexedDB auto-save |
| **2** | ✅ F7 Voxel 3D | Hecho | 32 capas, cuevas, dungeons |
| **3** | ✅ F8 Ecosistemas | Hecho | Estaciones, crecimiento, frutos, clima |
| **4** | F9 Criaturas IA | 4-6h | Pathfinding A*, alimentar/domar, seguir |
| **5** | F10 Audio 3D | 5-8h | PannerNode, eco, música dinámica |
| **6** | F13 Hidrología | 4-6h | Cascadas, foam, flora acuática, oleaje |
| **7** | F11 Portales | 3-5h | Shader distorsión, fade, hub |
| **8** | F15 Social | 5-8h | Chat burbujas, comandos, co-op |
| **9** | F17 Arq. | 4-6h | Plazas, puentes, variedad por bioma |
| **10** | F18 VR | 10-15h | WebXR, manos, 72fps |
| **11** | F19 Modding | 8-10h | Biomas TOML, plugins, blueprints |
| **12** | F20 Optimizar | continuo | LOD, workers, PWA, i18n |
