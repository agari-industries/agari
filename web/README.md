# Agari Web

A web-based Riichi Mahjong calculator powered by the [Agari](https://github.com/ryblogs/agari) scoring engine via WebAssembly.

## Features

- 🀄 **Interactive Tile Selection** - Click tiles to build your hand with a visual palette
- 📊 **Real-time Shanten** - See shanten updates as you build your hand
- 🎯 **Complete Scoring** - Calculate han, fu, and payment with full yaku breakdown
- 🎨 **Beautiful UI** - SVG-rendered tiles with smooth animations
- ⚡ **Instant Calculation** - All processing done client-side via WebAssembly
- 📱 **Responsive Design** - Works on desktop and mobile

## Development

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://rustup.rs/) (for building WASM)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)

### Setup

1. Install dependencies:
   ```bash
   npm install
   ```

2. Build the WASM module (from the repo root):
   ```bash
   wasm-pack build crates/agari-wasm --target web --out-dir ../../web/src/lib/wasm
   ```

3. Start the dev server:
   ```bash
   npm run dev
   ```

4. Open [http://localhost:5173](http://localhost:5173) in your browser

### Building for Production

```bash
npm run build
```

The built files will be in the `dist/` directory, ready for static hosting.

## Project Structure

```
web/
├── public/
│   └── favicon.svg       # Site favicon
├── src/
│   ├── lib/
│   │   ├── components/   # Svelte components
│   │   │   ├── Tile.svelte           # SVG tile renderer
│   │   │   ├── TilePalette.svelte    # Tile selection grid
│   │   │   ├── HandDisplay.svelte    # Hand visualization
│   │   │   ├── ContextOptions.svelte # Game options
│   │   │   └── ScoreResult.svelte    # Score display
│   │   ├── wasm/         # WASM bindings (generated)
│   │   └── agari.ts      # TypeScript API wrapper
│   ├── App.svelte        # Main application
│   ├── app.css           # Global styles
│   └── main.ts           # Entry point
├── index.html
├── package.json
├── vite.config.ts
└── tsconfig.json
```

## Tech Stack

- **[Svelte 5](https://svelte.dev/)** - UI framework with runes
- **[Vite](https://vitejs.dev/)** - Build tool
- **[Tailwind CSS 4](https://tailwindcss.com/)** - Styling
- **[TypeScript](https://www.typescriptlang.org/)** - Type safety
- **[wasm-bindgen](https://rustwasm.github.io/wasm-bindgen/)** - Rust-JS interop

## Deployment

The site can be deployed to any static hosting service:

- **GitHub Pages** - Use the `gh-pages` branch
- **Vercel** - Connect your repo for automatic deployments
- **Netlify** - Drag and drop the `dist/` folder

## License

MIT - Same as the main Agari project