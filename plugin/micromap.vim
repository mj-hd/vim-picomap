if exists('g:micromap_loaded')
	finish
endif

let g:micromap_loaded = 1

augroup MicromapUpdate
	autocmd!
	autocmd VimResized * call micromap#update()
	autocmd WinEnter * call micromap#update()
	autocmd WinLeave * call micromap#update()
	autocmd WinNew * call micromap#update()
augroup END

let g:micromap_winblend = 30
let g:micromap_sync_interval = 100

let s:bin_suffix = has('win32') ? '.exe' : ''
let s:env = 'debug'
let g:micromap_bin = '/target/' . s:env . '/vim-micromap' . s:bin_suffix

if s:env == 'debug'
	nnoremap md :call micromap#debug()<cr>
	nnoremap ms :call micromap#show()<cr>
	nnoremap mh :call micromap#hide()<cr>

endif
