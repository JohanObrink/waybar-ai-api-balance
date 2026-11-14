# waybar-ai-api-balance

A small Waybar module for monitoring balances across AI API providers.

Currently only **Moonshot** is implemented. The goal is to keep the code small,
test-driven, and easy to extend with more providers later.

## Usage

Set your Moonshot API key:

```bash
export MOONSHOT_API_KEY="sk-..."
```

Run the binary:

```bash
waybar-ai-api-balance
```

It prints Waybar JSON:

```json
{"text":"󰚩 $42.15","tooltip":"Moonshot API\nCash: $42.15\nVoucher: $0.00","class":"moonshot-balance"}
```

## Waybar configuration

```jsonc
"custom/ai-api-balance": {
    "exec": "waybar-ai-api-balance",
    "interval": 60
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
