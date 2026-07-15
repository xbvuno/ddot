# ⚡ ddot

A high-performance, modular image manipulation library written in Rust. Designed from the ground up for speed and flexibility, `ddot` provides a fast core engine, a dynamic CLI for automation, and high-quality WebAssembly (WASM) bindings for modern web applications.

---

## 📦 Architecture Overview

The repository is structured as a Cargo workspace containing the following crates:

*   **`core`**: The pure-Rust image processing engine. It defines standard image models, a dynamic filter architecture, palette generation algorithms, dithering, image resizing, and GPU compute dispatch.
*   **`cli`**: A robust command-line application that allows introspecting, generating schemas, and chaining multiple filters sequentially using JSON pipelines. Supports CPU and GPU (wgpu/WebGPU) backends.
*   **`wasm`**: High-performance WASM bindings that bridge the core Rust library directly to browser/Node.js environments via `wasm-bindgen`.
*   **`core-macros`**: Procedural macros (`#[derive(Filter)]` and `#[derive(FilterParams)]`) used to declare filters and auto-generate parameter schemas and validation logic.

---

## 🛠️ The Core Engine (`ddot-core`)

The core engine contains several dedicated modules:

### 1. Image Model
*   `Image`: A basic wrapper holding `width`, `height`, and a raw `Vec<u8>` pixel buffer (`[r, g, b, a]`).
*   `Color`: Represents individual RGB/RGBA pixels.

### 2. Filters
Built-in filters are declared via macros and registered in the `FILTERS` slice:
*   **`adjustment`**: Handles global levels adjustments. Supports **CPU + GPU**.
    *   `gamma` (range: `0.3..3.0`, default: `1.0`)
    *   `blacks` (range: `-0.5..0.5`, default: `0.0`)
    *   `whites` (range: `-0.5..0.5`, default: `0.0`)
    *   `contrast` (range: `-100..500`, default: `0`)
    *   `saturation` (range: `0.0..10.0`, default: `1.0`)
    *   `hue` (range: `-PI..PI`, default: `0.0`)
*   **`noise`**: Deterministic procedural noise filter. Supports **CPU + GPU**.
    *   `coverage` (range: `0.0..1.0`, default: `0.0`)
    *   `intensity` (range: `0.0..1.0`, default: `0.0`)
    *   `saturation` (range: `0.0..1.0`, default: `0.0`)
*   **`gaussian_blur`**: Separable 1D Gaussian blur. Supports **CPU + GPU**.
    *   `radius` (range: `0..50`, default: `1`)
    *   `sigma` (range: `0.1..20.0`, default: `1.0`)
*   **`kawase_blur`**: Fast multi-pass Kawase blur. Supports **CPU + GPU**.
    *   `iterations` (range: `1..10`, default: `3`)
    *   `offset` (range: `0.5..5.0`, default: `1.0`)

### 3. Palette Generation
Exposes algorithms to analyze an image and generate optimized color palettes:
*   **Median Cut**: Standard box-splitting palette reducer.
*   **Octree**: Tree-based quantization.
*   **K-Means**: Iterative clustering for highly precise palette generation.

### 4. Dithering
Applies error-diffusion or ordered dithering based on a color palette. **11 algorithms** available:

| Algorithm | GPU Support | Description |
|---|---|---|
| Floyd-Steinberg | ❌ CPU only | Classic error-diffusion |
| Atkinson | ❌ CPU only | Soft error-diffusion (1/8 factor) |
| Stucki | ❌ CPU only | High-quality error-diffusion |
| Burkes | ❌ CPU only | Simplified Stucki variant |
| Sierra | ❌ CPU only | 3-row error diffusion |
| Sierra Two Row | ❌ CPU only | 2-row Sierra variant |
| Sierra Lite | ❌ CPU only | Lightweight Sierra |
| Jarvis-Judice-Ninke (JJN) | ❌ CPU only | Wide kernel diffusion |
| Bayer | ✅ CPU + GPU | Ordered threshold matrix |
| Random | ✅ CPU + GPU | Randomized ordered dithering |
| OnlyPalette | ❌ CPU only | Nearest palette color, no diffusion |

### 5. Transform
*   **Resize**: Nearest Neighbor downscaler optimized for fast rendering pipelines.
*   **Crop**: Crop an image to a given bounding box (`top`, `left`, `right`, `bottom`).

### 6. GPU Backend
The core supports a wgpu/WebGPU compute backend with automatic CPU fallback:
*   Filters declare a WGSL compute shader via `gpu_shader()`.
*   The CLI uses `wgpu` (native); the WASM target uses the browser's WebGPU API.
*   On `auto` mode: GPU is attempted, and falls back to CPU silently if unavailable.
*   On `gpu` mode: Falls back to CPU with a warning if WebGPU is unavailable.

---

## 💻 CLI Usage (`ddot-cli`)

The CLI offers a dynamic pipeline interface for automating image operations.

### Installation

```bash
cargo build --release --bin ddot-cli
```

The binary will be at `target/release/ddot-cli` (or `ddot-cli.exe` on Windows).

### 1. List Available Filters
List all registered filters, their parameters, data types, default values, and valid ranges:
```bash
ddot-cli list
```

Example output:
```
Available filters:

  * Filter: adjustment
    Parameters:
      - gamma (float, default: 1, range: 0.3..3)
      - blacks (float, default: 0, range: -0.5..0.5)
      - whites (float, default: 0, range: -0.5..0.5)
      - contrast (integer, default: 0, range: -100..500)
      - saturation (float, default: 1, range: 0..10)
      - hue (float, default: 0, range: -3.1415927..3.1415927)

  * Filter: noise
    Parameters:
      - coverage (float, default: 0, range: 0..1)
      - intensity (float, default: 0, range: 0..1)
      - saturation (float, default: 0, range: 0..1)
```

### 2. Generate JSON Schema
Export a clean JSON template of a single pipeline step for a specific filter. Since it writes directly to `stdout`, you can redirect/pipe it to a configuration file:
```bash
# Output template to terminal
ddot-cli schema adjustment

# Direct redirection to a JSON file
ddot-cli schema adjustment > step.json
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
Apply one or more filters in sequence to an input image. The `--pipeline` flag accepts either an inline JSON array string or a path to a JSON pipeline file. Select the backend with `--backend` (or `-b`):

| Backend | Behavior |
|---|---|
| `auto` (default) | Tries GPU, falls back to CPU silently |
| `cpu` | Always uses CPU |
| `gpu` | Uses GPU; warns and falls back to CPU if unavailable |

**Using inline JSON and CPU backend:**
```bash
ddot-cli apply input.jpg -o output.png \
  --pipeline '[{"name": "adjustment", "settings": {"saturation": 0.0}}]' \
  --backend cpu
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
    "name": "gaussian_blur",
    "settings": {
      "radius": 3,
      "sigma": 1.5
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

Execute it on the GPU:
```bash
ddot-cli apply input.jpg -o output.png --pipeline pipeline.json --backend gpu
```

> **Note:** If `--output` / `-o` is omitted, the output is saved next to the input as `<stem>_ddot.png` (e.g., `photo.jpg` → `photo_ddot.png`).

---

## 🌐 WebAssembly API (`ddot-wasm`)

Compile the WASM target using `wasm-pack`:
```bash
# Run the PowerShell build script
./build_wasm.ps1
```

Here is a guide on how to load and use the WASM library in JavaScript/TypeScript:

### 1. Loading & Manipulating Images
```javascript
import { Image } from "ddot-wasm";

// Initialize from browser ImageData
const canvas = document.getElementById("canvas");
const ctx = canvas.getContext("2d");
const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);

const wasmImage = new Image(imageData);

// Accessors
console.log(wasmImage.width, wasmImage.height);
const pixels = wasmImage.pixels; // Uint8Array copy of the pixel buffer

// Deep copy
const copy = wasmImage.clone();

// Render back to canvas
ctx.putImageData(wasmImage.toImageData(), 0, 0);
```

### 2. Applying Filters Dynamically
```javascript
import { Filters } from "ddot-wasm";

// Get all available filters as an Array with named properties
const filterHandles = Filters.getFilters();

// Access by name (named property on the array)
const adjustment = filterHandles.adjustment;
const gaussianBlur = filterHandles.gaussian_blur;

// Or get just filter names
const names = Filters.getFilterNames(); // ["adjustment", "noise", "gaussian_blur", ...]

// Inspect a filter handle
console.log(adjustment.name);           // "adjustment"
console.log(adjustment.supportsGpu);    // true
console.log(adjustment.backendSupport); // "cpuandgpu"
console.log(adjustment.getParams());    // Array of param descriptors

// Apply a filter asynchronously (await required)
await adjustment.apply(wasmImage, {
  gamma: 1.5,
  saturation: 0.8,
  contrast: 10
});
```

### 3. GPU Backend
```javascript
import { Filters } from "ddot-wasm";

const noise = Filters.getFilters().noise;

// Check if a filter supports GPU
console.log(noise.supportsGpu); // true

// Access the WGSL shader source (if available)
console.log(noise.gpuShader);

// apply() dispatches to GPU via WebGPU when available.
// Falls back to CPU automatically.
await noise.apply(wasmImage, { coverage: 0.3, intensity: 0.05 });
```

### 4. Generating Color Palettes
```javascript
import { Palettes } from "ddot-wasm";

const generators = Palettes.Generators; // { MedianCut, Octree, Kmeans }

// Median Cut — params: { n_of_colors: int (2..256, default 16) }
const palette = generators.MedianCut.calculate(wasmImage, { n_of_colors: 16 });

// Octree — params: { n_of_colors: int (2..256, default 16) }
const palette2 = generators.Octree.calculate(wasmImage, { n_of_colors: 8 });

// K-Means — params: { n_of_colors, max_iterations, tolerance }
const palette3 = generators.Kmeans.calculate(wasmImage, {
  n_of_colors: 16,
  max_iterations: 10,  // range: 1..100, default: 10
  tolerance: 0.1       // range: 0.00001..1.0, default: 0.1
});

// Read colors as array of { r, g, b, a } objects
console.log(palette.colors);
```

### 5. Dithering
```javascript
import { Dithering } from "ddot-wasm";

// Access algorithms as a named object
const algs = Dithering.Algorithms;
// Keys: FloydSteinberg, Atkinson, Stucki, Burkes, Sierra,
//       SierraTwoRow, SierraLite, Jjn, Bayer, Random, OnlyPalette

// Or get as an Array
const algArray = Dithering.getAlgorithms();

// Error-diffusion algorithms — params: { amount: float (0.0..1.0, default 1.0) }
algs.FloydSteinberg.apply(wasmImage, palette, { amount: 1.0 });
algs.Atkinson.apply(wasmImage, palette, { amount: 0.8 });
algs.Stucki.apply(wasmImage, palette, { amount: 1.0 });
algs.Burkes.apply(wasmImage, palette, { amount: 1.0 });
algs.Sierra.apply(wasmImage, palette, { amount: 1.0 });
algs.SierraTwoRow.apply(wasmImage, palette, { amount: 1.0 });
algs.SierraLite.apply(wasmImage, palette, { amount: 1.0 });
algs.Jjn.apply(wasmImage, palette, { amount: 1.0 });

// Bayer — params: { amount: float (0..1), matrixScale: int (1..8, default 1) }
algs.Bayer.apply(wasmImage, palette, { amount: 1.0, matrixScale: 2 }); // GPU accelerated

// Random — params: { amount: float (0..1, default 0.65), seed: float (0..100, default 1.0) }
algs.Random.apply(wasmImage, palette, { amount: 0.65, seed: 42 }); // GPU accelerated

// OnlyPalette — no params (maps each pixel to the nearest palette color)
algs.OnlyPalette.apply(wasmImage, palette, {});
```

### 6. Transform
```javascript
import { Transform } from "ddot-wasm";

// Resize (Nearest Neighbor)
const resized = Transform.Resize(wasmImage, { width: 300, height: 200 });

// Crop
const cropped = Transform.Crop(wasmImage, {
  top: 10,
  left: 10,
  right: 290,
  bottom: 190
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
