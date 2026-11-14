const fs = require('fs');
const path = require('path');
const svgicons2svgfont = require('svgicons2svgfont');
const svg2ttf = require('svg2ttf');

// Mono SVGs from @lobehub/icons-static-svg (lobehub.com/icons).
// The `file` slug is the LobeHub icon id; the `name` is the provider id this
// widget uses in its config and CSS class names.
const icons = [
  { slug: 'kimi', name: 'kimi', codepoint: 0xE000 },
  { slug: 'deepseek', name: 'deepseek', codepoint: 0xE001 },
  { slug: 'claude', name: 'claude', codepoint: 0xE002 },
  { slug: 'ollama', name: 'ollama', codepoint: 0xE003 },
  { slug: 'openrouter', name: 'openrouter', codepoint: 0xE004 },
];

const iconBase = path.join(__dirname, '..', 'node_modules/@lobehub/icons-static-svg/icons');

const sources = icons.map(icon => {
  const file = path.join(iconBase, `${icon.slug}.svg`);
  if (!fs.existsSync(file)) {
    throw new Error(`Missing icon SVG: ${file}`);
  }
  return {
    ...icon,
    stream: fs.createReadStream(file),
  };
});

const fontStream = new svgicons2svgfont.SVGIcons2SVGFontStream({
  fontName: 'AI Provider Logos',
  fontId: 'ai-provider-logos',
  normalize: true,
  centerHorizontally: true,
  fontHeight: 1000,
  // Center glyphs on the text baseline. The default descent of 0 puts the
  // baseline at the bottom of the em square, making every icon float above
  // the surrounding text.
  descent: 500,
  log: () => {},
});

let svgFont = '';
fontStream.on('data', chunk => svgFont += chunk);
fontStream.on('end', () => {
  const ttf = svg2ttf(svgFont, {});
  const output = path.join(__dirname, '..', 'assets/ai-provider-logos.ttf');
  fs.writeFileSync(output, Buffer.from(ttf.buffer));
  console.log('Generated', output);
});

sources.forEach(s => {
  const glyph = Object.assign(s.stream, {
    metadata: {
      unicode: [String.fromCodePoint(s.codepoint)],
      name: s.name,
    },
  });
  fontStream.write(glyph);
});
fontStream.end();
