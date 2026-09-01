# Quickstart: Warm-To-Cold Restart

```text
fragcap deep-capture <TARGET> --launch --restart-warm
```

fragcap displays the uncertain image-name observation and the bounded plan. After confirmation, close the named application through its own normal Exit or Quit control. fragcap does not stop it. When the declared images are absent, fragcap resolves and prepares the cold launch again and asks for final launch authorization.

For an unattended environment where another trusted operator or supervisor performs normal shutdown, `--yes` pre-confirms both prompts. The deadline remains finite. Calibration cannot be combined with this option.
