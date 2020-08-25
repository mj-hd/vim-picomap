## vim-picomap
visualize your code with overview like Visual Studio's minimap. inspired by [nanomap](https://github.com/hisaknown/nanomap.vim).

**🚧 under development 🚧**

![Kapture 2020-08-26 at 00 29 54](https://user-images.githubusercontent.com/6854255/91194887-85102e80-e733-11ea-91c6-070b53d7bfe8.gif)


### TODO

- [ ] highlight search results
- [ ] highlight selected range
- [ ] mouse support

## Requirements

- Neovim

## Installation

### vim-plug

```vim
Plug 'mj-hd/vim-picomap', { 'do': 'bash install.sh' }
```

## Usage

TODO

## Configuration

### Global variables

- `g:picomap_autostart`: (default: 1) show picomap when vim starts
- `g:picomap_sync_interval`: (default: 100) interval to sync picomap window (ms)
- `g:picomap_gitgutter`: (default: 1) enable visualizing gitgutter's hunk
- `g:picomap_coc`: (default: 1) enable visualizing coc's diagnostics
- `g:picomap_winbled`: (default: 30) opacity of picomap window

## Contribution

Welcome PRs!

## Licence

[NYSL](http://www.kmonos.net/nysl/)
