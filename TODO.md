# WORLDS — Plan de Desarrollo Pendiente

Priorizado por impacto/dependencias. Cada fase lista archivos a modificar y tareas concretas.

---

## 1. F5 — Persistencia (Save/Load en IndexedDB)
**Impacto:** ⭐⭐⭐⭐ | **Depende de:** — | **Tamaño:** 2-3h

- [ ] Migrar `localStorage` a `IndexedDB` (almacenamiento más grande y estructurado)
- [ ] Serializar: posición, waypoints, descubrimientos, seed, parámetros, día, inventario, logros, codex, bloques colocados
- [ ] Ranuras de guardado (3 slots) con nombre, timestamp, screenshot thumbnail
- [ ] Auto-save cada 30s + Save manual
- [ ] Carga automática al iniciar (con diálogo "nuevo mundo" vs "continuar")
- [ ] Load game desde el menú principal

**Archivos:** `app.rs`, `state/mod.rs`, `engine/mod.rs`, `index.html`

---

## 2. F7 — Terreno Voxel 3D (Cuevas reales)
**Impacto:** ⭐⭐⭐⭐⭐ | **Depende de:** F6 | **Tamaño:** 15-20h
> ⚠️ FASE MAS PESADA — dividir en sub-fases:

### 2a. Estructura de chunks 3D
- [ ] Cambiar `ChunkData` de heightmap 2D a grid tridimensional (16×64×16)
- [ ] Algoritmo de tallado 3D (noise 3D para cuevas, cavernas, túneles)
- [ ] Face culling: solo renderizar caras visibles entre bloques sólidos y aire
- [ ] Transición suave superficie↔subterráneo (blending de colores)

### 2b. Bloques individuales
- [ ] Tipos de bloque: tierra, piedra, carbón, hierro, oro, diamante, lava, agua subterránea
- [ ] Break animation (partículas al romper)
- [ ] Drop de items al romper
- [ ] Iluminación de antorcha (vertex lighting dinámico)

### 2c. Features subterráneas
- [ ] Acuíferos subterráneos
- [ ] Lagos de lava en profundidad (zona Magma)
- [ ] Dungeons/Dungeon rooms generadas proceduralmente
- [ ] Cristales gigantes (Crystal zone bajo tierra)

**Archivos:** `chunk.rs` (reforma mayor), `terrain.rs`, `engine/mod.rs`, `minerals.rs`, `three_bridge.js`, nuevo `engine/voxel.rs`

---

## 3. F8 — Ecosistemas Dinámicos
**Impacto:** ⭐⭐⭐⭐ | **Depende de:** F1 | **Tamaño:** 8-10h

- [ ] Sistema de estaciones (4 estaciones con duración configurable)
- [ ] Cambio de color de follaje por estación
- [ ] Crecimiento de árboles (semilla → brote → joven → adulto) con tick semanal
- [ ] Flores y polinización (insectos voladores)
- [ ] Frentes meteorológicos: sistemas de lluvia que se desplazan por el mapa
- [ ] Animales de bioma (mariposas, peces, aves) como partículas especiales

**Archivos:** `vegetation.rs`, `particles.rs`, `engine/mod.rs`, `terrain.rs`, `creatures.rs`, `app.rs`

---

## 4. F9 — Criaturas con IA
**Impacto:** ⭐⭐⭐⭐⭐ | **Depende de:** F7, F8 | **Tamaño:** 8-12h

- [ ] Pathfinding A* sobre grid del terreno
- [ ] Comportamientos: deambular, huir, alimentarse, seguir
- [ ] Spawn condicional: tipo de criatura según bioma, hora, clima
- [ ] Animaciones (idle, walk, run, attack) con morph targets o rotación de partes
- [ ] Criaturas de rescate ocultas en estructuras con recompensa
- [ ] Interacción: click para examinar, alimentar, domar
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
**Impacto:** ⭐⭐⭐⭐ | **Depende de:** F7 | **Tamaño:** 6-8h

- [ ] Ríos: detectar cauces naturales desde altura hasta water_level (erosion simulation)
- [ ] Puentes sobre ríos (conectar caminos F17)
- [ ] Oleaje en costa (vertex displacement en shader de agua)
- [ ] Flora acuática: algas, corales, kelp (mesh instances en zona acuática)
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
| **1** | F5 Persistencia | 2-3h | Sin save no hay progreso real |
| **2** | F7 Voxel 3D | 15-20h | Desbloquea cuevas, minería real, dungeons |
| **3** | F8 Ecosistemas | 8-10h | Da vida al mundo |
| **4** | F9 Criaturas IA | 8-12h | El mundo se siente habitado |
| **5** | F10 Audio 3D | 5-8h | Inmersión masiva con poco código |
| **6** | F13 Hidrología | 6-8h | Ríos + oleaje transforman el agua |
| **7** | F11 Portales | 3-5h | Mejora viajes entre mundos |
| **8** | F15 Social | 5-8h | Multiplayer completo |
| **9** | F17 Arq. | 4-6h | Civilización procedural |
| **10** | F18 VR | 10-15h | Experiencia inmersiva |
| **11** | F19 Modding | 8-10h | Contenido infinito |
| **12** | F20 Optimizar | continuo | Pulido final |
