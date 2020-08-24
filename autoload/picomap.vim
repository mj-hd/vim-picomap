let s:ch = 0
let s:timer = 0
let s:debug_bufnr = 0
let s:server_started = 0
let s:server_retries = 0

let s:dir = expand('<sfile>:p:h')

function! s:on_error(id, data, event) abort
	if s:debug_bufnr == 0
		let s:debug_bufnr = bufadd('picomap-debug')
	endif

	call appendbufline(s:debug_bufnr, '$', join(a:data, "\n"))
endfunction

function! s:on_exit(id, data, event) abort
	call s:timer_stop()

	" restart the server with unexpected exit
	if s:server_started && !g:picomap_leaving
		let s:server_retries += 1

		let s:ch = 0

		if s:server_retries > 1
			let s:server_retries = 0
			let s:server_started = 0
			echoerr 'server exited unexpectedly'
			return
		endif

		call s:start_server()
		call s:timer_start()
	endif
endfunction

" start the server and store channel id to s:ch
function! s:start_server() abort
	if s:ch > 0
		return v:true
	endif

	let s:ch = jobstart([fnamemodify(s:dir, ':h') . g:picomap_bin], { 'on_stderr': funcref('s:on_error'), 'on_exit': funcref('s:on_exit'), 'rpc': v:true })

	if s:ch == 0
		echoerr 'server could not be started'
		return v:false
	elseif s:ch == -1
		echoerr 'server is not executable'
		return v:false
	endif

	let s:server_started = 1

	return v:true
endfunction

" sync the server and set next sync timer
function! s:sync(timer) abort
	let s:timer = 0

	if s:ch == 0
		return
	endif

	if g:picomap_leaving
		return
	endif

	if g:picomap_coc
		call CocAction('fillDiagnostics', bufnr('%'))
	endif

	let l:diags = getloclist(win_getid())

	let l:changes = []

	if g:picomap_gitgutter
		let l:changes = GitGutterGetHunks()
	endif

	call rpcnotify(s:ch, 'sync', l:diags, l:changes)

	let s:server_retries = 0

	call s:timer_start()
endfunction

" start sync timer
function! s:timer_start()
	if s:timer > 0
		return
	endif

	let s:timer = timer_start(g:picomap_sync_interval, funcref('s:sync'), {})
endfunction

" stop sync timer
function! s:timer_stop()
	if s:timer > 0
		call timer_stop(s:timer)
		let s:timer = 0
	endif
endfunction

function! picomap#init() abort
	if g:picomap_autostart
		call picomap#show()
	endif
endfunction

function! picomap#show() abort
	let l:success = s:start_server()

	call rpcnotify(s:ch, 'show')

	if l:success
		call s:timer_start()
	endif
endfunction

function! picomap#resize()
	if s:ch > 0
		call rpcnotify(s:ch, 'resize')
	endif
endfunction

function! picomap#hide()
	call s:timer_stop()
	if s:ch > 0
		call rpcnotify(s:ch, 'close')
	endif
endfunction

function! picomap#debug() abort
	if s:debug_bufnr == 0
		let s:debug_bufnr = bufadd('picomap-debug')
		call setbufvar(s:debug_bufnr, '&swapfile', 0)
		call setbufvar(s:debug_bufnr, '&buftype', 'nofile')
		call setbufvar(s:debug_bufnr, '&undolevels', -1)
	endif
	execute ':buffer ' . s:debug_bufnr
endfunction

function! picomap#restart() abort
	if s:ch > 0
		call s:timer_stop()
		" trigger restart
		call jobstop(s:ch)
	endif
endfunction
