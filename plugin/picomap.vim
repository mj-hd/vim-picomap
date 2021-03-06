if exists('g:picomap_loaded')
	finish
endif

let g:picomap_loaded = 1
let g:picomap_leaving = 0

augroup PicomapUpdate
	autocmd!
	autocmd VimResized * call picomap#resize()
	autocmd WinEnter * call picomap#resize()
	autocmd WinLeave * call picomap#resize()
	autocmd WinNew * call picomap#resize()
augroup END

augroup Picomap
	autocmd!
	autocmd VimLeavePre * let g:picomap_leaving = 1
	autocmd VimEnter * call picomap#init()
augroup END

let g:picomap_autostart = 1
let g:picomap_winblend = 30
let g:picomap_sync_interval = 100
let g:picomap_gitgutter = 1
let g:picomap_coc = 1

let s:bin_suffix = has('win32') ? '.exe' : ''
let s:env = 'debug'
let g:picomap_bin = '/target/' . s:env . '/vim-picomap' . s:bin_suffix

if s:env == 'debug'
	nnoremap md :call picomap#debug()<cr>
	nnoremap ms :call picomap#show()<cr>
	nnoremap mh :call picomap#hide()<cr>
	nnoremap mr :call picomap#restart()<cr>
endif
