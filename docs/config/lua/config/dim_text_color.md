---
tags:
  - color
  - appearance
---
# `dim_text_color`

{{since('nightly')}}

Sets the foreground color used whenever a cell requests the half-intensity (SGR 2)
attribute. If this value is not provided, wezterm will dim the existing
foreground color by a fixed amount.

```lua
local wezterm = require 'wezterm'
local config = {}

config.colors = {
  dim_text_color = '#808080',
}

return config
```

Any color specification accepted by other palette fields (named colors, `#RRGGBB`,
HSL/HWB/HSV forms, etc.) may be used here.
