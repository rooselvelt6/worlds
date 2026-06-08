#!/bin/bash
set -euo pipefail

PROJECT="$(cd "$(dirname "$0")/.." && pwd)"
CLIENT_DIST="$PROJECT/client/dist"
SERVER_ASSETS="$PROJECT/server/assets"

# 1. Sync three_bridge.js
cp "$PROJECT/client/three_bridge.js" "$SERVER_ASSETS/three_bridge.js"
echo "[deploy] three_bridge.js synced"

# 2. Find the WASM hash from the built files
WASM_JS=$(ls "$CLIENT_DIST"/worlds-app-*.js 2>/dev/null | head -1)
if [ -z "$WASM_JS" ]; then
    echo "[deploy] ERROR: no worlds-app-*.js found in $CLIENT_DIST"
    echo "[deploy] Run 'trunk build --release --config client/Trunk.toml' first"
    exit 1
fi

BASENAME=$(basename "$WASM_JS" .js)          # worlds-app-<hash>
HASH="${BASENAME#worlds-app-}"               # just the hash

# 3. Copy WASM files to server
cp "$CLIENT_DIST/$BASENAME.js" "$SERVER_ASSETS/"
cp "$CLIENT_DIST/$BASENAME"__bg.wasm "$SERVER_ASSETS/" 2>/dev/null || cp "$CLIENT_DIST/${BASENAME}_bg.wasm" "$SERVER_ASSETS/"
echo "[deploy] WASM files copied (hash: $HASH)"

# 4. Remove old WASM files (anything with a different hash)
for f in "$SERVER_ASSETS"/worlds-app-*.js "$SERVER_ASSETS"/worlds-app-*.wasm; do
    fb=$(basename "$f")
    if [ "$fb" != "$BASENAME.js" ] && [ "$fb" != "${BASENAME}_bg.wasm" ]; then
        rm -f "$f"
        echo "[deploy] removed old: $fb"
    fi
done

# 5. Update hash in server/assets/index.html
sed -i "s/worlds-app-[a-f0-9]*\.js/$BASENAME.js/g" "$SERVER_ASSETS/index.html"
sed -i "s/worlds-app-[a-f0-9]*_bg\.wasm/${BASENAME}_bg.wasm/g" "$SERVER_ASSETS/index.html"
echo "[deploy] index.html hash updated"

# 6. Sync server index → client dist (for trunk serve consistency)
cp "$SERVER_ASSETS/index.html" "$CLIENT_DIST/index.html"
echo "[deploy] index.html synced to client/dist"

# 7. Verify all critical files exist and serve correctly
echo ""
echo "[deploy] === Verification ==="
for f in "index.html" "three_bridge.js" "$BASENAME.js" "${BASENAME}_bg.wasm" "worker.js" "three/three.module.js" "three/postprocessing/EffectComposer.js" "three/postprocessing/SSRPass.js" "three/postprocessing/SSAOPass.js" "three/postprocessing/SMAAPass.js" "three/csm/CSM.js"; do
    if [ -f "$SERVER_ASSETS/$f" ]; then
        echo "  OK  $f"
    else
        echo "  MISSING  $f"
    fi
done

echo ""
echo "[deploy] Done. Hash: $HASH"
