# Wasm Imager

A pure WebAssembly library for building/synthesizing embedded OS disk images (GPT / MBR / Raw) directly in the browser or Node.js without backend processing.

## Installation

### For Web/NPM Projects

Build the package using `wasm-pack`:

```bash
wasm-pack build --target web
```

Then import it in your project:

```javascript
import init, { generate_layout } from "./pkg/wasm_imager.js";

await init();
// Use generate_layout...
```

## Schema Example

```json
{
  "table": "gpt",
  "disk_id": "11223344-5566-7788-9900-AABBCCDDEEFF",
  "partitions": [
    {
      "id": "boot",
      "type": "0FC63DAF-8483-4772-8E79-3D69D8477DE4",
      "offset": 1048576,
      "size": 16777216,
      "hidden": false
    }
  ]
}
```
