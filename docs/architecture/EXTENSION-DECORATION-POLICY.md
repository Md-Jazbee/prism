# Decoration policy (P8)

Setting: `prism.decorations.enabled` (**default: false**).

| Decoration | When | Noise control |
|---|---|---|
| Slice highlight (whole line) | After `prism.slice` | Cleared on next clear / disable |
| Hotspot overview ruler | Opt-in future | Off unless enabled |
| Ambiguity `?` after-content | Opt-in | Off by default |

Do not enable a large gutter icon set by default — users disable noisy extensions. Prefer panel + peek over inline chrome.
