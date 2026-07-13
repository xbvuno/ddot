# ⚡ ddot

A high-performance, modular image manipulation library written in Rust. Designed from the ground up for speed and flexibility, `ddot` provides a fast core engine, a dynamic CLI for automation, and high-quality WebAssembly (WASM) bindings for modern web applications.

---

## 📦 Architecture Overview

The repository is structured as a Cargo workspace containing the following crates:

*   **`core`**: The pure-Rust image processing engine. It defines standard image models, a dynamic filter architecture, palette generation algorithms, dithering, and image resizing.
*   **`cli`**: A robust command-line application that allows introspecting, generating schemas, and chaining multiple filters sequentially using JSON pipelines.
*   **`wasm`**: High-performance WASM bindings that bridge the core Rust library directly to browser/Node.js environments.
*   **`core-macros`**: Procedural macros (`#[derive(Filter)]` and `#[derive(FilterParams)]`) used to declare filters and auto-generate parameter schemas and validation logic.

---

## 🛠️ The Core Engine (`ddot-core`)

The core engine contains several dedicated modules:

### 1. Image Model
*   `Image`: A basic wrapper holding `width`, `height`, and a raw `Vec<u8>` pixel buffer (`[r, g, b, a]`).
*   `Color`: Represents individual RGB/RGBA pixels.

### 2. Filters
Built-in filters are declared via macros and registered in the `FILTERS` slice:
*   **`adjustment`**: Handles global levels adjustments.
    *   `gamma` (range: `0.3..3.0`, default: `1.0`)
    *   `blacks` (range: `-0.5..0.5`, default: `0.0`)
    *   `whites` (range: `-0.5..0.5`, default: `0.0`)
    *   `contrast` (range: `-100..500`, default: `0`)
    *   `saturation` (range: `0.0..10.0`, default: `1.0`)
    *   `hue` (range: `-PI..PI`, default: `0.0`)
*   **`noise`**: Deterministic procedural noise filter.
    *   `coverage` (range: `0.0..1.0`, default: `0.0`)
    *   `intensity` (range: `0.0..1.0`, default: `0.0`)
    *   `saturation` (range: `0.0..1.0`, default: `0.0`)

### 3. Palette Generation
Exposes algorithms to analyze an image and generate optimized color palettes:
*   **Median Cut**: Standard box-splitting palette reducer.
*   **Octree**: Tree-based quantization.
*   **K-Means**: Iterative clustering for highly precise palette generation.

### 4. Dithering
Applies error-diffusion or ordered dithering based on a color palette:
*   *Algorithms*: Floyd-Steinberg, Atkinson, Stucki, Burkes, Sierra, Bayer, and OnlyPalette.

### 5. Resizing
*   Nearest Neighbor downscaler optimized for fast rendering pipelines.

---

## 💻 CLI Usage (`ddot-cli`)

The CLI offers a dynamic pipeline interface for automating image operations.

### 1. List Available Filters
List all registered filters, their parameters, data types, default values, and valid ranges:
```bash
cargo run --bin ddot-cli -- list
```

### 2. Generate JSON Schema
Export a clean JSON template of a single pipeline step for a specific filter. Since it writes directly to `stdout`, you can redirect/pipe it to a configuration file:
```bash
# Output template to terminal
cargo run --bin ddot-cli -- schema adjustment

# Direct redirection to a JSON file
cargo run --bin ddot-cli -- schema adjustment > step.json
```

**Output example (`step.json`):**
```json
{
  "name": "adjustment",
  "settings": {
    "blacks": 0.0,
    "contrast": 0,
    "gamma": 1.0,
    "hue": 0.0,
    "saturation": 1.0,
    "whites": 0.0
  }
}
```

### 3. Apply a Filter Pipeline
Apply one or more filters in sequence to an input image. The `--pipeline` argument accepts either an inline JSON array string or a path to a JSON pipeline file. You can select the backend using the `--backend` (or `-b`) flag:

**Using inline JSON and CPU backend:**
```bash
cargo run --bin ddot-cli -- apply input.jpg -o output.png --pipeline '[{"name": "adjustment", "settings": {"saturation": 0.0}}]' --backend cpu
```

**Using a pipeline file and GPU acceleration:**
Create a pipeline configuration (`pipeline.json`):
```json
[
  {
    "name": "adjustment",
    "settings": {
      "gamma": 1.2,
      "contrast": 15
    }
  },
  {
    "name": "noise",
    "settings": {
      "coverage": 0.3,
      "intensity": 0.05
    }
  }
]
```
Execute it on the GPU (non-supported filters like `noise` will automatically fallback to CPU with a warning):
```bash
cargo run --bin ddot-cli -- apply input.jpg -o output.png --pipeline pipeline.json --backend gpu
```

---

## 🌐 WebAssembly API (`ddot-wasm`)

Compile the WASM target using `wasm-pack`:
```bash
# Run the PowerShell build script
./build.ps1
```

Here is a guide on how to load and use the WASM library in JavaScript/TypeScript:

### 1. Loading & Manipulating Images
```javascript
import { Image } from "ddot-wasm";

// Initialize WasmImage from browser ImageData
const canvas = document.getElementById("canvas");
const ctx = canvas.getContext("2d");
const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);

const wasmImage = new Image(imageData);
```

### 2. Applying Filters Dynamically
```javascript
import { Filters, Backend } from "ddot-wasm";

// Get list of all available filters
const filterHandles = Filters.getFilters();
const adjustmentFilter = filterHandles.adjustment; // Retrieve by name

// Check supported backends ("cpuonly" or "cpuandgpu")
console.log(adjustmentFilter.backendSupport);

// Apply adjustment filter asynchronously (await is required)
// The optional third argument specifies the backend: Backend.Auto (default), Backend.Cpu, Backend.Gpu
// (You can also pass strings directly: "auto", "cpu", "gpu")
await adjustmentFilter.apply(wasmImage, {
  gamma: 1.5,
  saturation: 0.8,
  contrast: 10
}, Backend.Gpu);

// Render back to canvas
const updatedImageData = wasmImage.toImageData();
ctx.putImageData(updatedImageData, 0, 0);
```

### 3. Generating Color Palettes & Dithering
```javascript
import { Palettes, Dithering } from "ddot-wasm";

// 1. Get generator and compute a 16-color palette
const generators = Palettes.Generators;
const octree = generators.Octree;
const palette = octree.calculate(wasmImage, { n_of_colors: 16 });

// Read colors array: [{ r: ..., g: ..., b: ..., a: ... }, ...]
console.log(palette.colors);

// 2. Dither the image using Floyd-Steinberg and the generated palette
const ditherers = Dithering.Algorithms;
const floyd = ditherers.FloydSteinberg;
floyd.apply(wasmImage, palette, { amount: 1.0 });
```

### 4. Resizing Images
```javascript
import { Transform } from "ddot-wasm";

const resizedImage = Transform.Resize(wasmImage, {
  width: 300,
  height: 200
});
```

---

## 🧪 Testing & Validation

Run unit tests for all crates:
```bash
cargo test
```

Run clippy checks:
```bash
cargo clippy --all-targets
```
