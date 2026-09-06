const CACHE='nta-home-calculator-v1';
const ASSETS=['./','./index.html','./engine.js','./manifest.webmanifest'];
self.addEventListener('install',event=>event.waitUntil(caches.open(CACHE).then(cache=>cache.addAll(ASSETS))));
self.addEventListener('fetch',event=>event.respondWith(caches.match(event.request).then(hit=>hit||fetch(event.request).catch(()=>caches.match('./index.html')))));
