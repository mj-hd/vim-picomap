if exists('g:picomap_loaded')
	finish
endif

let g:picomap_loaded = 1

augroup MicromapUpdate
	autocmd!
	autocmd VimResized * call picomap#update()
	autocmd WinEnter * call picomap#update()
	autocmd WinLeave * call picomap#update()
	autocmd WinNew * call picomap#update()
augroup END

let g:picomap_winblend = 30
let g:picomap_sync_interval = 100

let s:bin_suffix = has('win32') ? '.exe' : ''
let s:env = 'debug'
let g:picomap_bin = '/target/' . s:env . '/vim-picomap' . s:bin_suffix

if s:env == 'debug'
	nnoremap md :call picomap#debug()<cr>
	nnoremap ms :call picomap#show()<cr>
	nnoremap mh :call picomap#hide()<cr>
endif
