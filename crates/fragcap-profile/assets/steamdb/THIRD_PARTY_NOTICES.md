# Third-party notice: SteamDB FileDetectionRuleSets

This directory vendors `rules.ini` from the SteamDB `FileDetectionRuleSets`
project, used by fragcap's technology-detection surface to recognize game
engines, anti-cheat systems, SDKs, emulators, containers, and launchers from an
install directory's file paths.

- Source: https://github.com/SteamDatabase/FileDetectionRuleSets
- Pinned commit: 243cf741921d2c8fd6b844f83831edf4692cf788
- License: MIT

The file is vendored verbatim from the pinned commit (normalized only to UTF-8
without BOM and LF line endings). Its integrity is recorded in `rules.lock.json`.
fragcap applies the ruleset to detect technologies; it does not modify traffic,
processes, or the detected software.

## License

MIT License

Copyright (c) 2021 SteamDB

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
