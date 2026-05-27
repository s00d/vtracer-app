# VTracer App

[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-blue)](https://github.com/s00d/vtracer-app)
[![License](https://img.shields.io/github/license/s00d/vtracer-app)](https://github.com/s00d/vtracer-app/blob/master/LICENSE)
[![Release](https://img.shields.io/github/v/release/s00d/vtracer-app)](https://github.com/s00d/vtracer-app/releases)
[![Downloads](https://img.shields.io/github/downloads/s00d/vtracer-app/total)](https://github.com/s00d/vtracer-app/releases)
[![Issues](https://img.shields.io/github/issues/s00d/vtracer-app)](https://github.com/s00d/vtracer-app/issues)
[![Stars](https://img.shields.io/github/stars/s00d/vtracer-app?style=social)](https://github.com/s00d/vtracer-app/stargazers)

VTracer App is a desktop tool for converting raster images to SVG.

It is a user-friendly wrapper around [`vtracer`](https://github.com/visioncortex/vtracer):  
`vtracer` does the tracing, this app provides the UI, workflow, preview, and batch tools.

![VTracer App Screenshot](./img.png)

## What You Can Do

- Convert a single image to SVG
- Convert multiple images in batch
- Tune tracing parameters with instant preview
- Compare original vs result (slider / side-by-side / overlay)
- Select crop area for saved output
- Save and reuse presets
- Keep conversion history
- Enable output post-processing:
  - trim transparent margins
  - optimize SVG

## Tracing Controls (from `vtracer`)

- Presets: `bw`, `poster`, `photo`
- Color mode: `color`, `bw`
- Hierarchical mode: `stacked`, `cutout`
- Path mode: `pixel`, `polygon`, `spline`
- Detail/noise controls:
  - `filterSpeckle`
  - `colorPrecision`
  - `gradientStep`
- Shape controls:
  - `cornerThreshold`
  - `segmentLength`
  - `spliceThreshold`
  - `pathPrecision`
  - `maxIterations`

## Why This App

`vtracer` is powerful, but many users want a desktop workflow with visual feedback and batch operations.  
This app is focused exactly on that.
