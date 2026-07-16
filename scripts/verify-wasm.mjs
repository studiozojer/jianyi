// Node smoke test: WASM cast of the known seed must match native's primary=45.
import init, { cast } from "../pkg/jianyi.js";
import { readFile } from "node:fs/promises";
const bytes = await readFile(new URL("../pkg/jianyi_bg.wasm", import.meta.url));
await init({ module_or_path: bytes });
const out = JSON.parse(cast(JSON.stringify({ timestamp: "2026-06-12T21:51:54+00:00" })));
if (out.primary.number !== 45) throw new Error(`expected 45, got ${out.primary.number}`);
console.log("WASM determinism OK: primary =", out.primary.number);
