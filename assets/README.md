# AI Provider Logos font

`ai-provider-logos.ttf` is a minimal icon font built from [Simple Icons](https://simple-icons.org/)
(CC0-1.0). It contains brand logos for the providers supported by this widget.

To rebuild the font after adding a provider:

```bash
npm install
npm run build-font
```

## Included icons and codepoints

| Provider   | Simple Icons slug | Codepoint |
|------------|-------------------|-----------|
| Kimi       | `kimi`            | `U+E000`  |
| MoonshotAI | `moonshotai`      | `U+E001`  |
| DeepSeek   | `deepseek`        | `U+E002`  |
| Claude     | `claude`          | `U+E003`  |
| Anthropic  | `anthropic`       | `U+E004`  |
| Ollama     | `ollama`          | `U+E005`  |

To install the font locally:

```bash
mkdir -p ~/.local/share/fonts
cp assets/ai-provider-logos.ttf ~/.local/share/fonts/
fc-cache -f ~/.local/share/fonts
```
