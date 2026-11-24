#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

PKG_NAME="${NPM_PKG_NAME:-tex2math}"
PUBLISH_REGISTRY="${NPM_PUBLISH_REGISTRY:-}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --github)
      PKG_NAME="@twn39/tex2math"
      PUBLISH_REGISTRY="https://npm.pkg.github.com"
      shift
      ;;
    --name)
      PKG_NAME="$2"
      shift 2
      ;;
    --registry)
      PUBLISH_REGISTRY="$2"
      shift 2
      ;;
    -h|--help)
      echo "Usage: $0 [--github] [--name <pkg-name>] [--registry <url>]"
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      exit 1
      ;;
  esac
done

# Prefer rustup toolchain when available (avoids Homebrew rustc lacking wasm32 std).
if [ -d "$HOME/.cargo/bin" ]; then
  export PATH="$HOME/.cargo/bin:$PATH"
fi

if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "error: wasm-pack is required (cargo install wasm-pack)" >&2
  exit 1
fi

rm -rf pkg
mkdir -p pkg

echo "==> Building Web / ESM target (pkg/)..."
wasm-pack build --release --target web --out-dir pkg -- --features wasm

echo "==> Building Node.js target (pkg/nodejs/)..."
wasm-pack build --release --target nodejs --out-dir pkg/nodejs -- --features wasm

# Remove wasm-pack .gitignore files so `npm pack` / `npm publish` includes pkg assets,
# and drop duplicated README/LICENSE inside nodejs/.
rm -f pkg/.gitignore pkg/nodejs/.gitignore pkg/nodejs/LICENSE pkg/nodejs/README.md

# Ensure Node treats pkg/nodejs/*.js as CommonJS even when pkg/package.json has "type": "module".
cat > pkg/nodejs/package.json <<'EOF'
{
  "type": "commonjs"
}
EOF

# Provide a Node ESM entrypoint that supports both direct named imports and `await init()`.
cat > pkg/nodejs/index.mjs <<'EOF'
import cjs from "./tex2math.js";

export const {
  convert_batch,
  convert_to_mathml,
  convert_to_mathml_with_options,
  parse_to_ast,
  wasm_supports_command,
} = cjs;

export function initSync() {
  return undefined;
}

export default async function init() {
  return undefined;
}
EOF

cp pkg/tex2math.d.ts pkg/nodejs/index.d.mts

# Merge metadata into a universal package.json supporting Node.js (ESM + CJS) and Web (browser + bundlers).
PKG_NAME="$PKG_NAME" PUBLISH_REGISTRY="$PUBLISH_REGISTRY" node -e '
const fs = require("fs");
const pkg = JSON.parse(fs.readFileSync("pkg/package.json", "utf8"));
pkg.name = process.env.PKG_NAME;
pkg.type = "module";
pkg.main = "./nodejs/tex2math.js";
pkg.module = "./tex2math.js";
pkg.types = "./tex2math.d.ts";
pkg.files = [
  "tex2math.js",
  "tex2math_bg.wasm",
  "tex2math.d.ts",
  "tex2math_bg.wasm.d.ts",
  "nodejs/package.json",
  "nodejs/index.mjs",
  "nodejs/index.d.mts",
  "nodejs/tex2math.js",
  "nodejs/tex2math.d.ts",
  "nodejs/tex2math_bg.wasm",
  "nodejs/tex2math_bg.wasm.d.ts",
  "README.md",
  "LICENSE"
];
pkg.exports = {
  ".": {
    "node": {
      "types": "./nodejs/index.d.mts",
      "import": "./nodejs/index.mjs",
      "require": "./nodejs/tex2math.js",
      "default": "./nodejs/tex2math.js"
    },
    "types": "./tex2math.d.ts",
    "default": "./tex2math.js"
  },
  "./web": {
    "types": "./tex2math.d.ts",
    "default": "./tex2math.js"
  },
  "./nodejs": {
    "types": "./nodejs/index.d.mts",
    "import": "./nodejs/index.mjs",
    "require": "./nodejs/tex2math.js",
    "default": "./nodejs/tex2math.js"
  }
};
if (process.env.PUBLISH_REGISTRY) {
  pkg.publishConfig = { registry: process.env.PUBLISH_REGISTRY };
}
pkg.sideEffects = ["./snippets/*"];
fs.writeFileSync("pkg/package.json", JSON.stringify(pkg, null, 2) + "\n");
'

echo "==> Universal JS/WASM package (${PKG_NAME}) ready in pkg/"
