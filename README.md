<div align="center">

<img align="center" width="128px" src="src-tauri/icons/icon.png" />
<h1 align="center"><b>Qopy</b></h1>

<p align="center">
  <b>Forked by <a href="https://github.com/Compromisee">Compromisee</a></b>
  ·
  from <a href="https://github.com/0pandadev/qopy">PandaDev/Qopy</a>
</p>

The fixed and simple clipboard manager for both Windows and Linux.

<a href="https://github.com/0PandaDEV/Qopy/releases/download/v0.3.3/Qopy-0.3.3_x64.msi">
  <img src="./public/windows.png"> Windows (x64)
</a>
•
<a href="https://github.com/0PandaDEV/Qopy/releases/download/v0.3.3/Qopy-0.3.3_arm64.msi">
  Windows (arm64)
</a>
<br>
<a href="https://github.com/0PandaDEV/Qopy/releases/download/v0.3.3/Qopy-0.3.3.deb">
  <img src="./public/linux.png"> Linux (deb)
</a>
•
<a href="https://github.com/0PandaDEV/Qopy/releases/download/v0.3.3/Qopy-0.3.3.rpm">
  Linux (rpm)
</a>
•
<a href="https://github.com/0PandaDEV/Qopy/releases/download/v0.3.3/Qopy-0.3.3.AppImage">
  Linux (AppImage)
</a>
<br>
<a href="https://github.com/0PandaDEV/Qopy/releases/download/v0.3.3/Qopy-0.3.3_silicon.dmg">
  <img src="./public/apple.png"> macOS (Silicon)
</a>
•
<a href="https://github.com/0PandaDEV/Qopy/releases/download/v0.3.3/Qopy-0.3.3_intel.dmg">
  macOS (Intel)
</a>
<br>
<br>
<sup>Nightly releases can be found <a href="https://github.com/0PandaDEV/qopy/actions/workflows/build.yml">here</a> </sup>

</div>

[discord »](https://discord.gg/invite/PEX37vvWyU)

> \[!IMPORTANT]
>
> **Star this project**, You will receive all release notifications from GitHub without any delay \~ ⭐️

<details>
  <summary><kbd>Star History</kbd></summary>
  <a href="https://starchart.cc/0PandaDEV/Qopy">
    <picture>
      <img width="100%" src="https://starchart.cc/0PandaDEV/Qopy.svg?variant=adaptive">
    </picture>
  </a>
</details>

![ziit](https://ziit.app/api/public/badge/cm98il90n0000o52c3my0bf5p/qopy)

## 📋 What is Qopy

Qopy is a fixed clipboard manager designed as a simple alternative to the standard clipboard on Windows. It aims to provide a faster, more reliable experience while providing an extensive set of features compared to its Windows counterpart.

This branch is maintained by **Compromisee** as a fork of **PandaDev/Qopy**, and includes the reliability fixes and extra features listed below.

## ✨ Features

- **Reliable recording of every new copy** — the clipboard monitor runs fully panic-proof code (no more event-loop crashes that silently stopped saving), the programmatic-paste guard self-clears if it ever gets stuck, favicon fetches are time-boxed, and the list re-syncs with the database on every focus so nothing ever goes missing.
- **Reliable pasting of any previous item** — the selection is tracked by item id, so it can no longer be reset or shifted by list rebuilds, window focus changes, or duplicate clipboard events.
- **Glitch prevention system** — duplicate clipboard monitor events are filtered out (content fingerprinting), paste operations are guarded against re-entrancy, keyboard listeners are de-duplicated, and the hotkey is ignored while a programmatic paste is in flight.
- **Color tagging** — right-click any entry to open a custom context menu and tag it with one of 8 colors; the entry's row is highlighted in that color. Right-clicking anywhere else in the app no longer shows the OS context menu.
- **Type pills** — every entry shows a small pill with its type (`txt`, `png`, `url`, `hex`, `json`, `py`, `svg`, ...). Code entries get a pill color-coded uniquely per language, plus a large decorative language monogram that fades in from the left edge without cutting into the text.
- **Fixed language detection** — JSON/config files are no longer mislabeled as bash: the detector checks structural JSON first and keywords now match on word boundaries (`"fi"` inside `"identifier"` no longer counts as a bash keyword). Covered by regression tests.
- **Scroll containment** — the fixed 750×474 window can no longer be scrolled out of place: the page itself is locked and every inner scroller stops chaining at its edges.
- **Content type filters** — quick chips under the search bar (All / Text / Code / Link / Image / Color / File) to narrow the history list; can be hidden from Settings.
- **History retention** — Settings → History has a slider that keeps the newest N items (100–5000, default 1000); pinned items always survive, and the database is trimmed on startup and after every new entry. A live "N saved" counter shows the current size.
- **Clear search** — a × button next to the search bar clears the active query and restores the full list (without closing the app).
- **Code highlighting** — code snippets are detected (Python, Rust, JS/TS, Go, Java, C/C++, C#, Bash, SQL, HTML, CSS, PHP, Ruby, Swift, Kotlin, JSON, YAML, Markdown and more) and highlighted in the preview pane with [highlight.js](https://highlightjs.org/); the detection is deliberately aggressive so it fires on most real snippets, and the detected language is shown as a badge over the preview.
- **Themes** — pick from Ember, Midnight, Forest, Crimson, and Paper (light) themes in Settings; your choice is remembered.
- **Settings icon** — open Settings straight from the main window with the gear icon (⚙️).
- **Pin items** — pin frequently used entries to a "Pinned" group at the top of the list (hover an entry → pin icon).
- **Per-entry actions** — hover any entry (or press <kbd>⌘K</kbd> / <kbd>Ctrl K</kbd>) for quick actions: copy to clipboard without pasting, pin/unpin, and delete.
- **Delete entries** — remove single entries from history (hover → trash icon, or <kbd>Delete</kbd>).
- **Clear all history** — from the History section in Settings (with confirm step).
- **Empty states & feedback** — friendly messages when there is nothing to show, and toasts for copy/pin/delete actions.

## 🚧 Roadmap
- [x] [Setup guide](https://github.com/0PandaDEV/Qopy/blob/main/GET_STARTED.md)
- [ ] Sync Clipboard across devices https://github.com/0PandaDEV/Qopy/issues/8
- [x] Settings https://github.com/0PandaDEV/Qopy/issues/2
- [x] Metadata for copied items https://github.com/0PandaDEV/Qopy/issues/5
- [x] Code highlighting https://github.com/0PandaDEV/Qopy/issues/7
- [ ] Streamshare integration https://github.com/0PandaDEV/Qopy/issues/4
- [x] Content type filter https://github.com/0PandaDEV/Qopy/issues/16
- [ ] Preview for copied files https://github.com/0PandaDEV/Qopy/issues/15
- [ ] Convert files to other formats https://github.com/0PandaDEV/Qopy/issues/17
- [x] Option for custom keybind https://github.com/0PandaDEV/Qopy/issues/3
- [x] macOS Support https://github.com/0PandaDEV/Qopy/issues/13

<sup>If you have ideas for features to include, please write a feature request [here](https://github.com/0pandadev/Qopy/issues).</sup>

## 📦 Concepts

Here you can see a few concepts these might not be implemented:

![Clipboard](https://github.com/user-attachments/assets/45a44a13-6ebd-4f2d-84d2-55178e303a54)
![Settings](https://github.com/user-attachments/assets/bff5456a-f413-4e62-a43d-22c8e453aa87)


## ❤️ Donations & Support

Qopy is open-source and free to use. I appreciate donations to support ongoing development and improvements. Your contributions are voluntary and help me enhance the app for everyone.

<a href="https://buymeacoffee.com/pandadev_"><img src="https://img.shields.io/badge/Buy_Me_A_Coffee-FFDD00?style=for-the-badge&logo=buy-me-a-coffee&logoColor=black"/></a>

## ⌨️ Local development

You can use GitHub Codespaces for online development:

[![][codespaces-shield]][codespaces-link]

Or to get Qopy set up on your machine, you'll need to have Rust and bun installed. Then, follow these steps:

```zsh
git clone https://github.com/0pandadev/Qopy.git
cd Qopy
bun i
bun dev
```

> \[!TIP]
>
> If you are interested in contributing code, feel free to check out the [Issues](https://github.com/0pandadev/Qopy/issues) section.

## 🔨 Building for production

To build for production simply execute:

```zsh
bun build
```

> \[!NOTE]
>
> Don't worry, it will fail at the end because it can not detect a Private key, but the installer files will be generated regardless of that.
> 
> You can find them in `src-tauri/target/release/bundle`.

## 📝 License

Qopy is licensed under AGPL-3. See the [LICENSE file](./LICENCE) for more information.

[codespaces-link]: https://codespaces.new/0pandadev/Qopy
[codespaces-shield]: https://github.com/codespaces/badge.svg
