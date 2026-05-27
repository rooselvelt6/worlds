# WORLDS — Roadmap de Realismo (Fases R1–R10)

> 10 fases para llevar WORLDS a realismo total.
> Priorizadas por impacto visual y dependencias técnicas.

---

## R1 — Terreno con Normales Suaves + Detail Mesh
**Impacto:** ⭐⭐⭐⭐⭐ | **Esfuerzo:** 4-6h | **Depende de:** —
- Normales suaves por vértice (promediar normales de triángulos adyacentes en grid n+1 × n+1)
- Detail mesh: segunda malla transparente con textura de detalle (arena/rocas/hojarasca) superpuesta con blending
- Parallax mapping para micro-relieve en lugar de subdivisión geométrica
- **Archivos:** `chunk.rs`, `three_bridge.js`, `terrain.rs`

---

## R2 — Océano Realista con SSR y Shoreline
**Impacto:** ⭐⭐⭐⭐⭐ | **Esfuerzo:** 6-8h | **Depende de:** R1
- Gerstner waves completas: desplazar XZ + Y por vértice, normales correctas desde la función de onda
- Screen-Space Reflections (SSR) con `SSRPass` de Three.js
- Shoreline blending: vertex alpha cerca de costa → transición agua→terreno
- Underwater post-processing: tinte azul, distorsión, burbujas
- **Archivos:** `three_bridge.js`, `mod.rs`, `chunk.rs`, `bridge.rs`

---

## R3 — Atmósfera + Nubes Volumétricas
**Impacto:** ⭐⭐⭐⭐⭐ | **Esfuerzo:** 5-7h | **Depende de:** —
- Rayleigh + Mie scattering en shader de cielo (Three.js `Sky` o custom)
- Sol con corona, glow solar y halo
- Nubes volumétricas: billboards alpha-blended con textura procedural
- Capas: cirrus (altas), cumulus (medias), stratus (bajas)
- Viento desplaza nubes horizontalmente
- **Archivos:** `three_bridge.js`, `bridge.rs`

---

## R4 — Post-Processing Cinematográfico Completo
**Impacto:** ⭐⭐⭐⭐ | **Esfuerzo:** 4-6h | **Depende de:** R3
- LUT 3D: color grading por bioma con transiciones suaves
- Depth of Field (`BokehPass` o `DepthOfFieldPass`)
- Vignette dinámica (intensa en cuevas, sutil de día)
- Lens flare al mirar al sol
- Adaptive exposure (auto ojo humano al entrar/salir de cuevas)
- Film grain sutil en entornos oscuros
- **Archivos:** `three_bridge.js`, `index.html`, `bridge.rs`

---

## R5 — Vegetación Realista + GPU Instancing
**Impacto:** ⭐⭐⭐⭐ | **Esfuerzo:** 5-7h | **Depende de:** R1
- Billboards impostors: cross-quads texturados para copas lejanas
- `THREE.InstancedMesh`: 1 draw call por tipo en lugar de 120 por chunk
- Troncos cilíndricos, copas con múltiples planos alfa, ramas
- Sway por vértice en GPU (shader de viento)
- Detail grass: billboards de pasto (InstancedMesh, 1000+ blades por chunk)
- **Archivos:** `vegetation.rs`, `three_bridge.js`, `bridge.rs`, `chunk.rs`

---

## R6 — CSM + Sombras de Alta Calidad
**Impacto:** ⭐⭐⭐⭐ | **Esfuerzo:** 4-6h | **Depende de:** —
- Cascaded Shadow Maps (CSM): 3-4 cascadas con resolución progresiva
- Contact hardening: sombras más nítidas cerca del contacto
- PCSS (Percentage Closer Soft Shadows)
- Alpha-test shadow maps para vegetación
- **Archivos:** `three_bridge.js`, `index.html`, `bridge.rs`

---

## R7 — SSAO + Iluminación Global de Cuevas
**Impacto:** ⭐⭐⭐⭐ | **Esfuerzo:** 6-8h | **Depende de:** R6
- Screen Space Ambient Occlusion (SSAO) para profundidad en cuevas/esquinas
- Baked light maps para chunks subterráneos
- God rays (volumetric light) desde antorchas y claros
- Emissive glow real con bloom + halo volumétrico
- **Archivos:** `three_bridge.js`, `chunk.rs`, `bridge.rs`

---

## R8 — Audio HRTF + Reverb por Zona
**Impacto:** ⭐⭐⭐ | **Esfuerzo:** 4-5h | **Depende de:** —
- HRTF con `PannerNode` avanzado (coneInnerAngle, coneOuterAngle, rolloffFactor)
- IR diferentes por zona: cueva (larga), bosque (media), pradera (corta), dungeon (metálica)
- Wind audio modulado por altura y velocidad del viento
- Sonidos ambientales de fauna según bioma
- Water presence: intensidad de río/cascada basada en distancia real
- **Archivos:** `audio.rs`, `three_bridge.js`, `bridge.rs`

---

## R9 — Erosión y Tectónica de Placas
**Impacto:** ⭐⭐⭐⭐⭐ | **Esfuerzo:** 8-10h | **Depende de:** R1
- Placas tectónicas: 2-3 placas → montañas en bordes, valles en divergencia
- Erosión hidráulica + thermal erosion: barrancos, deltas, acantilados naturales
- Cuencas hidrográficas: flujo desde crestas hasta el mar, cauces realistas
- Sedimentación: valles fértiles, deltas en desembocaduras
- Plataforma continental, talud, llanura abisal
- **Archivos:** `terrain.rs`, `chunk.rs`, `engine/erosion.rs` (nuevo)

---

## R10 — Mundo Vivo: Partículas, Huellas, Destrucción
**Impacto:** ⭐⭐⭐⭐ | **Esfuerzo:** 6-8h | **Depende de:** R5, R7
- Micropartículas: polen, polvo, hojas secas, nieve levantada por viento
- Huellas al caminar/correr (decals o desplazamiento de vértices)
- Destrucción de terreno: mesh de superficie se deforma al explotar/minar
- Flora reactiva: arbustos y pasto se apartan al paso del jugador
- Meteoros/rocas que caen en montaña
- Estaciones visibles: nieve acumulada en invierno, barro en primavera
- **Archivos:** `particles.rs`, `chunk.rs`, `three_bridge.js`, `mod.rs`, `bridge.rs`, `terrain.rs`

---

## Resumen

| # | Fase | Impacto | Esfuerzo | Depende de |
|---|------|---------|----------|------------|
| R1 | Terreno Normales Suaves | ⭐⭐⭐⭐⭐ | 4-6h | — |
| R2 | Océano SSR + Shoreline | ⭐⭐⭐⭐⭐ | 6-8h | R1 |
| R3 | Atmósfera + Nubes | ⭐⭐⭐⭐⭐ | 5-7h | — |
| R4 | Post-Processing | ⭐⭐⭐⭐ | 4-6h | R3 |
| R5 | Vegetación Instancing | ⭐⭐⭐⭐ | 5-7h | R1 |
| R6 | CSM Sombras | ⭐⭐⭐⭐ | 4-6h | — |
| R7 | SSAO + Cuevas | ⭐⭐⭐⭐ | 6-8h | R6 |
| R8 | Audio HRTF | ⭐⭐⭐ | 4-5h | — |
| R9 | Erosión/Tectónica | ⭐⭐⭐⭐⭐ | 8-10h | R1 |
| R10 | Mundo Vivo | ⭐⭐⭐⭐ | 6-8h | R5, R7 |

**Total estimado:** 52-72 horas
