<script setup lang="ts">
const features = [
  "Live SVG preview while tuning settings",
  "Compare modes: slider, side-by-side, overlay",
  "Crop workflow (preview full image, apply crop on save)",
  "Batch conversion queue for folders of images",
  "Saved presets and conversion history",
  "Optional trim and SVG optimization on export",
];
</script>

<template>
  <div class="page">
    <header class="topbar">
      <div class="brand">
        <span class="logo-mark">V</span>
        <span>VTracer</span>
      </div>
      <nav class="links">
        <a href="https://github.com/s00d/vtracer-app" target="_blank" rel="noreferrer">GitHub</a>
        <a href="https://github.com/s00d/vtracer-app/releases" target="_blank" rel="noreferrer">Download</a>
      </nav>
    </header>

    <main>
      <section class="hero">
        <p class="badge">Desktop Wrapper for vtracer</p>
        <h1>Convert raster images to clean SVG fast</h1>
        <p class="sub">
          VTracer gives you a modern desktop workflow on top of
          <a href="https://github.com/visioncortex/vtracer" target="_blank" rel="noreferrer">vtracer</a>:
          visual tuning, real-time preview, batch conversion, and one-click export.
        </p>
        <div class="cta">
          <a class="btn primary" href="https://github.com/s00d/vtracer-app/releases" target="_blank" rel="noreferrer">
            Download builds
          </a>
          <a class="btn ghost" href="https://github.com/s00d/vtracer-app" target="_blank" rel="noreferrer">
            Source code
          </a>
        </div>
      </section>

      <section class="preview-card">
        <img
          src="https://raw.githubusercontent.com/s00d/vtracer-app/master/img.png"
          alt="VTracer app screenshot"
        />
      </section>

      <section class="features">
        <h2>Why this wrapper exists</h2>
        <p>
          <strong>vtracer</strong> is excellent at vectorization. VTracer adds the UX around it so you can
          work faster and ship cleaner SVG outputs without fighting CLI flags every time.
        </p>
        <ul>
          <li v-for="item in features" :key="item">{{ item }}</li>
        </ul>
      </section>

      <section class="tech">
        <h2>Built with</h2>
        <p>Tauri 2 · Rust · Vue 3 · TypeScript</p>
      </section>
    </main>
  </div>
</template>
