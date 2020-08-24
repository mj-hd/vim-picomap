## vim-picomap
visualize your code with overview like Visual Studio's minimap. inspired by [nanomap](https://github.com/hisaknown/nanomap.vim).

**🚧 under development 🚧**

![screenshot](https://user-images.githubusercontent.com/6854255/87242111-aa3b2d00-c464-11ea-96f5-3f7a0bd314e0.png)

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
