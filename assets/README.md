# AI Provider Logos font

`ai-provider-logos.ttf` is a minimal icon font built from the mono SVGs in
[@lobehub/icons-static-svg](https://www.npmjs.com/package/@lobehub/icons-static-svg)
([lobehub.com/icons](https://lobehub.com/icons)). LobeHub maintains accurate,
up-to-date brand logos for AI providers and models.

To rebuild the font after adding a provider:

```bash
npm install
npm run build-font
```

## Included icons and codepoints

| Provider   | LobeHub id  | Codepoint |
|------------|-------------|-----------|
| Kimi       | `kimi`      | `U+E000`  |
| DeepSeek   | `deepseek`  | `U+E001`  |
| Claude     | `claude`    | `U+E002`  |
| Anthropic  | `anthropic` | `U+E003`  |
| Ollama     | `ollama`    | `U+E004`  |
| OpenRouter | `openrouter`| `U+E005`  |

To install the font locally:

```bash
mkdir -p ~/.local/share/fonts
cp assets/ai-provider-logos.ttf ~/.local/share/fonts/
fc-cache -f ~/.local/share/fonts
```
