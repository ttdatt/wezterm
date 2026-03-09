# `ClearScrollback`

Clears the lines that have scrolled off the top of the viewport, resetting
the scrollbar thumb to the full height of the window.

{{since('20210203-095643-70a364eb')}}

Added a parameter that allows additionally clear the viewport:

```lua
local wezterm = require 'wezterm'
local act = wezterm.action

config.keys = {
  -- Clears only the scrollback and leaves the viewport intact.
  -- You won't see a difference in what is on screen, you just won't
  -- be able to scroll back until you've output more stuff on screen.
  -- This is the default behavior.
  {
    key = 'K',
    mods = 'CTRL|SHIFT',
    action = act.ClearScrollback 'ScrollbackOnly',
  },
  -- Clears the scrollback and viewport leaving the prompt line the new first line.
  {
    key = 'K',
    mods = 'CTRL|SHIFT',
    action = act.ClearScrollback 'ScrollbackAndViewport',
  },
  -- Clears the scrollback and viewport while preserving the current
  -- prompt when WezTerm can identify it.
  {
    key = 'K',
    mods = 'CTRL|SHIFT',
    action = act.ClearScrollback 'ScrollbackAndViewportKeepPrompt',
  },
  -- Clears the scrollback and viewport, and then sends CTRL-L to ask the
  -- shell to redraw its prompt
  {
    key = 'K',
    mods = 'CTRL|SHIFT',
    action = act.Multiple {
      act.ClearScrollback 'ScrollbackAndViewport',
      act.SendKey { key = 'L', mods = 'CTRL' },
    },
  },
}
```

`ScrollbackAndViewportKeepPrompt` prefers OSC 133 semantic prompt markers to
find the active prompt block in the viewport. With shell integration enabled,
this allows multi-line prompt content such as working directory or right prompt
information to remain visible after clearing.

If the pane does not provide semantic prompt markers, WezTerm also applies a
small fallback heuristic for prompt layouts such as fish + Starship, where the
prompt context is on one row and the editable prompt is on the next row.

If WezTerm cannot identify prompt context, it falls back to the same behavior
as `ScrollbackAndViewport`.
