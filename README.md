# waybar-ai-api-balance

A small Waybar module for monitoring balances across AI API providers.

Currently only **Moonshot** is implemented. The goal is to keep the code small,
test-driven, and easy to extend with more providers later.

## Usage

Set your Moonshot API key:

```bash
export MOONSHOT_API_KEY="sk-..."
waybar-ai-api-balance
```

It prints Waybar JSON showing the available balance:

```json
{"text":"$42.15","tooltip":"Moonshot API\nAvailable: $42.15\nCash: $42.15\nVoucher: $0.00","class":"moonshot-balance"}
```

Click the module to open a desktop notification with the full balance details.

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
    "format": "{}",
    "tooltip": true,
    "on-click": "/home/johan/.local/bin/waybar-ai-api-balance-launch --notify"
}
```

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
