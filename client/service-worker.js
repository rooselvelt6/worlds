const CACHE = "worlds-v5";
const PRECACHE_URLS = [
  "/",
  "/index.html",
  "/three_bridge.js",
  [
    "https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4",
    "sha384-AIH1kL7JmmReCxSwiSyaRNwzSFO7h4Ir4F/PO28EKezqFza1LwIMgEPLd83KwmZ4",
  ],
  [
    "https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.7.2/css/all.min.css",
    "sha384-nRgPTkuX86pH8yjPJUAFuASXQSSl2/bBUiNV47vSYpKFxHJhbcrGnmlYpYJMeD7a",
  ],
];

function verifyIntegrity(response, expectedHash) {
  if (!expectedHash) return true;
  return crypto.subtle.digest("SHA-384", response.clone().body)
    .then(hash => {
      const actual = btoa(String.fromCharCode(...new Uint8Array(hash)));
      return actual === expectedHash.split("-")[1];
    })
    .catch(() => true);
}

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches.open(CACHE).then(async (cache) => {
      for (const entry of PRECACHE_URLS) {
        const [url, integrity] = typeof entry === "string" ? [entry, null] : entry;
        try {
          const response = await fetch(url);
          if (response.ok) {
            if (integrity) {
              const valid = await verifyIntegrity(response, integrity);
              if (!valid) {
                console.warn(`SRI mismatch for ${url}, skipping cache`);
                continue;
              }
            }
            cache.put(url, response.clone());
          }
        } catch (e) {
          console.warn(`Failed to precache ${url}:`, e);
        }
      }
    })
  );
  self.skipWaiting();
});

self.addEventListener("activate", (event) => {
  event.waitUntil(clients.claim());
  event.waitUntil(
    caches.keys().then((keys) =>
      Promise.all(keys.filter((k) => k !== CACHE).map((k) => caches.delete(k)))
    )
  );
});

self.addEventListener("fetch", (event) => {
  if (event.request.method !== "GET") return;
  event.respondWith(
    caches.match(event.request).then((cached) => {
      const fetched = fetch(event.request).then((response) => {
        if (response.ok && response.type === "basic") {
          caches.open(CACHE).then((cache) => cache.put(event.request, response.clone()));
        }
        return response;
      }).catch(() => cached);
      return cached || fetched;
    })
  );
});
