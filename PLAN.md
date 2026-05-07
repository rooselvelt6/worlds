# WORLD'S - Plan de Mejoras en 10 Fases

## Estado Actual (Base)
- Motor Rust: 10 fórmulas matemáticas
- Frontend: 10 submundos conUI
- Rendering: Three.js chunksdinámicos
- Controles: WASD + mouse360°
- Cámara: Primera persona

---

## FASE 1: HUD y Crosshair (Semana1)
**Objetivo**: Mejorarlainterfazvisual sinromper lógica

### Cambios en index.html:
- [ ] Agregar crosshair (center screen)
- [ ] Agregar minimap (bottom-right)
- [ ] Mostrarbiome/coordenadas en HUD
- [ ] Indicador de FPS

###Archivos: index.html (CSS + HTML)
- Crosshair: centro de pantalla (CSS)
- Minimap:canvas 2D ocanvas3D
- Stats: actualizar en updateUI()

**Riesgo**: BAJO - Solo UI/CSS

---

## FASE 2: Texturas de Bloques (Semana2)
**Objetivo**: Agregar textures sin cambiar generación

### Nuevos archivos:
- assets/textures/grass.png
- assets/textures/stone.png
- assets/textures/sand.png
- assets/textures/water.png

### Cambios en index.html:
- [ ] TextureLoader de Three.js
- [ ] Mapeobiome → textura
- [ ] MeshLambertMaterial → MeshStandardMaterial

### Código nuevo (index.html):
```javascript
const textures = {};
function loadTextures() {
    const loader = new THREE.TextureLoader();
    textures.grass = loader.load('assets/textures/grass.png');
    // ... otros
}
```

**Riesgo**: BAJO - Agregar sin modificar getHeight()

---

## FASE 3: Skybox Dinámico (Semana3)
**Objetivo**: Cambiar elfondo según submundo

### Cambios en index.html:
- [ ] Array de colors por submundo
- [ ]scene.background actualizado
- [ ]scene.fog adaptive

### Código:
```javascript
const skyColors = {
    fractal_forest: 0x87ceeb,
    mandelbrot_realm: 0x1a0a2e,
    crystal_cavern: 0x2d1b4e,
    // ...
};
function updateSky() {
    scene.background = new THREE.Color(skyColors[params.subworld]);
}
```

**Riesgo**: BAJO - Solo cambio de color

---

## FASE 4: Ciclo Día/Noche (Semana4)
**Objetivo**: Luz dinámica según tiempo

### Cambios en index.html:
- [ ] Time tracker (0-1 cycle)
- [ ] Update sun position
- [ ] Update ambient light
- [ ] Toggle skycolor day/night

### Código:
```javascript
let dayTime = 0;
function updateDayNight() {
    dayTime = (dayTime + 0.001) % 1;
    const sunY = Math.sin(dayTime * Math.PI) * 80;
    sun.position.y = sunY;
    ambient.intensity = 0.3 + Math.max(0, Math.sin(dayTime * Math.PI)) * 0.5;
}
```

**Riesgo**: BAJO - Solo animaciones de luz

---

## FASE 5: Tercera Persona (Semana5)
**Objetivo**: Cámara detrás del jugador

### Cambios en index.html:
- [ ] Toggle first/thirdperson (key T)
- [ ] Thirdperson offset (behind + up)
- [ ] Avatar mesh visible en 3ra persona

### Código:
```javascript
let cameraMode = 'first'; // 'first' o 'third'
document.addEventListener('keydown', e => {
    if (e.key.toLowerCase() === 't') {
        cameraMode = cameraMode === 'first' ? 'third' : 'first';
    }
});
function updateCamera() {
    if (cameraMode === 'third') {
        const cx = pos.x - Math.sin(yaw) * 8;
        const cz = pos.z - Math.cos(yaw) * 8;
        camera.position.set(cx, pos.y + 4, cz);
        camera.lookAt(pos.x, pos.y, pos.z);
    }
}
```

**Riesgo**: MEDIO - Cambia cámara, easyollback

---

## FASE 6: Árboles y Vegetación (Semana6)
**Objetivo**: Agregar objetos3D al mundo

### Nuevas funciones en index.html:
- [ ] generateTree(x, y, z) → Mesh
- [ ] generateFlower(x, y, z) → Mesh
- [ ] spawnVegetation() en chunks

### Código:
```javascript
function createTree(x, y, z) {
    const trunk = new THREE.Mesh(
        new THREE.CylinderGeometry(0.3, 0.4, 3, 6),
        new THREE.MeshStandardMaterial({color: 0x8B4513})
    );
    const leaves = new THREE.Mesh(
        new THREE.ConeGeometry(2, 4, 8),
        new THREE.MeshStandardMaterial({color: 0x228B22})
    );
    leaves.position.y = 3.5;
    // Merge o Group
}
```

**Riesgo**: MEDIO - Nuevogeometry, sin cambiar terreno

---

## FASE 7: Minerales y Cuevas (Semana7)
**Objetivo**:添加矿物质ycuevas al motor Rust

### Cambios en shared/src/lib.rs:
- [ ] BlockType::Gold, BlockType::Diamond, BlockType::Coal
- [ ] generateCaveLayer() en WorldGenerator
- [ ] getOreAt() función nueva

### Código Rust:
```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum BlockType {
    // ... existente
    Coal,
    Gold,
    Diamond,
    Emerald,
}

impl BlockType {
    pub fn is_ore(&self) -> bool {
        matches!(self, BlockType::Coal | BlockType::Gold | BlockType::Diamond)
    }
}
```

**Riesgo**: MEDIO - Agregarenum, no cambiar existentes

---

## FASE 8: Sistema de Inventario (Semana8)
**Objetivo**: UI de inventario y guardar estado

### Cambios en index.html:
- [ ] HTML panel inventario
- [ ] Array inventory[] 
- [ ] Select block type (keys 1-9)
- [ ] Place/break blocks (click)

### Código:
```javascript
let inventory = ['grass', 'stone', 'wood', 'dirt'];
let selectedBlock = 0;
document.addEventListener('keydown', e => {
    if (e.key >= '1' && e.key <= '9') {
        selectedBlock = parseInt(e.key) - 1;
    }
});
canvas.addEventListener('click', () => {
    placeBlock(); // Raycasting
});
```

**Riesgo**: ALTO - Requiere cuidadodefault

---

## FASE 9: Guardar/Cargar Mundos (Semana9)
**Objetivo**: Persistenciajuga en localStorage

### Cambios en index.html:
- [ ] worldState = {pos, inventory, biome}
- [ ] saveWorld() → localStorage
- [ ] loadWorld() → localStorage
- [ ] Export/Import JSON

### Código:
```javascript
function saveWorld() {
    const state = {
        pos,
        inventory,
        params,
        seed: Date.now()
    };
    localStorage.setItem('worlds_save', JSON.stringify(state));
}

function loadWorld() {
    const saved = localStorage.getItem('worlds_save');
    if (saved) {
        const state = JSON.parse(saved);
        pos = state.pos;
        inventory = state.inventory;
    }
}
```

**Riesgo**: MEDIO - Solo localStorage

---

## FASE 10: Multiplayer Local (Semana10)
**Objetivo**: Dosjugadores en la misma máquina

### Cambios en index.html:
- [ ] Player 2 spawn (key P)
- [ ] Split-screen ocamera switch
- [ ] Inventory separar

### Código:
```javascript
let players = [
    {x: 50, y: 25, z: 50, color: 0x22d3ee},
    {x: 60, y: 25, z: 50, color: 0xf43f5e}
];
let activePlayer = 0;

document.addEventListener('keydown', e => {
    if (e.key.toLowerCase() === 'p') {
        activePlayer = 1 - activePlayer;
    }
});
```

**Riesgo**: BAJO - Solo input switch

---

## Resumen de Riesgos

| Fase | Riesgo | Rollback |
|------|--------|----------|
| 1 | BAJO | Easy |
| 2 | BAJO | Easy |
| 3 | BAJO | Easy |
| 4 | BAJO | Easy |
| 5 | MEDIO | Medium |
| 6 | MEDIO | Medium |
| 7 | MEDIO | Medium |
| 8 | ALTO | Hard |
| 9 | MEDIO | Medium |
| 10 | BAJO | Easy |

---

## Orden Recomendado

1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9 → 10

Comenzar por Fases 1-4 (BAJO riesgo) da confianza y mejoras visuales inmediatas.