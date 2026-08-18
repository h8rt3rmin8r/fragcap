# Quickstart: Installer npcap exit-dialog reconciliation

## What the user sees

Before this slice, the fragcap installer always ended with a pre-checked box:

```text
[x] Open the npcap download page (required before capturing traffic)
```

After it, the box is worded and defaulted to match reality:

```text
# On a machine that already has the npcap driver (installed with Wireshark):
[ ] Open the npcap download page (fragcap needs the npcap capture driver, if you
    do not already have it)

# On a machine without it:
[x] Open the npcap download page (fragcap needs the npcap capture driver, if you
    do not already have it)
```

Clicking Finish opens the vendor page only when the box is checked. The installer
still downloads, bundles, and installs no npcap.

## How it decides

The installer looks for `wpcap.dll` in the native system directory, the exact copy
`fragcap doctor` checks for and the one the live backend loads. Present means the
driver fragcap needs is already there, so the box is left unchecked; absent means it
pre-checks the option and, on Finish, opens the vendor download page.

## What is not covered by CI

`cargo xtask ci` does not build the MSI (candle/light run only in the release job),
so the detection and the checkbox default are verified by WiX-schema review here and
confirmed when the release job builds the installer.
