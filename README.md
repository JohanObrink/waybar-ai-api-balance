# waybar-ai-api-balance

A small Waybar module for monitoring balances across AI API providers.

The code is structured so adding a new provider means implementing the
`Provider` trait and adding it to the provider list. Currently only **Kimi**
(Moonshot API) is implemented.

## Usage

Set your Moonshot API key:

```bash
export MOONSHOT_API_KEY="sk-..."
waybar-ai-api-balance
```

It prints Waybar JSON with one block per provider:

```json
{"text":"<span font_family=\"AI Provider Logos\"></span> $42.15","tooltip":"Kimi\nAvailable: $42.15\nToday's consumption: $1.00\nTotal consumption: $5.00","class":"kimi-balance"}
```

Consumption is tracked locally by comparing the current balance against the
first reading and the first reading of the day. A top-up resets both counters.

Click the module to open a desktop notification with the full balance details.

## Install the icon font

The provider logos come from a custom icon font included in this repo:

```bash
mkdir -p ~/.local/share/fonts
cp assets/ai-provider-logos.ttf ~/.local/share/fonts/
fc-cache -f ~/.local/share/fonts
```

## Waybar configuration

Use the launcher script so the key can live in a file instead of the global
environment:

```bash
# ~/.config/waybar-ai-api-balance/env
MOONSHOT_API_KEY=sk-...
```

```jsonc
"custom/ai-api-balance": {
    "exec": "/home/johan/.local/bin/waybar-ai-api-balance-launch",
    "interval": 60,
    "return-type": "json",
    "escape": false,
    "format": "{}",
    "tooltip": true,
    "on-click": "/home/johan/.local/bin/waybar-ai-api-balance-launch --notify"
}
```

`escape: false` is required so Waybar renders the Pango markup that selects
the custom icon font for the logo.

## Configuration

| Environment variable | Description | Default |
|---|---|---|
| `MOONSHOT_API_KEY` | Your Moonshot API key. | required |
| `MOONSHOT_BASE_URL` | Moonshot API base URL. | `https://api.moonshot.ai/v1` |
| `MOONSHOT_CURRENCY` | Override currency (`USD` or `CNY`). | inferred from base URL |

## Development

```bash
cargo test
cargo build --release
```

To rebuild the logo font after adding a provider:

```bash
npm install
npm run build-font
```
