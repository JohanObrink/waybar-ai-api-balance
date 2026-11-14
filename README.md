# waybar-ai-api-balance

A small Waybar module for monitoring balances across AI API providers.

Adding a provider means implementing the `Provider` trait and registering it in
the provider list. Currently supported providers:

- **Kimi** (Moonshot API)
- **DeepSeek**

## Usage

Set your provider API keys and run the binary:

```bash
export MOONSHOT_API_KEY="sk-..."
export DEEPSEEK_API_KEY="sk-..."
waybar-ai-api-balance
```

By default it prints one Waybar JSON block per provider:

```json
{"text":"<span font_family=\"AI Provider Logos\"></span> $42.15","tooltip":"Kimi\nAvailable: $42.15\nToday's consumption: $1.00\nTotal consumption: $5.00","class":"kimi-balance"}
```

Consumption is tracked locally by comparing the current balance against the
first reading and the first reading of the day. A top-up resets both counters.

Clicking the module can open the provider account page or show a desktop
notification; wire it up with Waybar's `on-click` action.

## Install the icon font

The provider logos come from a custom icon font included in this repo:

```bash
mkdir -p ~/.local/share/fonts
cp assets/ai-provider-logos.ttf ~/.local/share/fonts/
fc-cache -f ~/.local/share/fonts
```

## Configuration

Configuration lives in `~/.config/waybar-ai-api-balance/config.toml`. Each
provider reads its API key from an environment variable by default; put secrets
in `~/.config/waybar-ai-api-balance/env` and use the launcher script to source
them without polluting the global environment.

```bash
# ~/.config/waybar-ai-api-balance/env
MOONSHOT_API_KEY=sk-...
DEEPSEEK_API_KEY=sk-...
```

## Waybar configuration

Use the launcher script so the keys can live in a file:

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

### One block per provider

Pass `--provider <id>` to show only one provider. This lets you place each
provider in its own Waybar module and bind a different click action to each:

```jsonc
"custom/ai-api-balance-kimi": {
    "exec": "/home/johan/.local/bin/waybar-ai-api-balance-launch --provider kimi",
    "interval": 60,
    "return-type": "json",
    "escape": false,
    "format": "{}",
    "tooltip": true,
    "on-click": "xdg-open https://platform.kimi.ai/console/account"
},
"custom/ai-api-balance-deepseek": {
    "exec": "/home/johan/.local/bin/waybar-ai-api-balance-launch --provider deepseek",
    "interval": 60,
    "return-type": "json",
    "escape": false,
    "format": "{}",
    "tooltip": true,
    "on-click": "xdg-open https://platform.deepseek.com/usage"
}
```

## Provider environment variables

| Environment variable | Description | Default |
|---|---|---|
| `MOONSHOT_API_KEY` | Your Moonshot API key. | required for Kimi |
| `MOONSHOT_BASE_URL` | Moonshot API base URL. | `https://api.moonshot.ai/v1` |
| `MOONSHOT_CURRENCY` | Override currency (`USD` or `CNY`). | inferred from base URL |
| `DEEPSEEK_API_KEY` | Your DeepSeek API key. | required for DeepSeek |
| `DEEPSEEK_BASE_URL` | DeepSeek API base URL. | `https://api.deepseek.com` |
| `DEEPSEEK_CURRENCY` | Override currency. | inferred from base URL |

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
