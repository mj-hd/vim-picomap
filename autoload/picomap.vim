let s:ch = 0
let s:timer = 0
let s:debug_bufnr = 0

let s:dir = expand('<sfile>:p:h')

function! s:error(id, data, event) abort
	if s:debug_bufnr == 0
		let s:debug_bufnr = bufadd('picomap-debug')
	endif

	call appendbufline(s:debug_bufnr, '$', join(a:data, ''))
endfunction

function! s:exit(id, data, event) abort
	call s:stop()
endfunction

function! s:start() abort
	if s:ch > 0
		return v:true
	endif

	let s:ch = jobstart([fnamemodify(s:dir, ':h') . g:picomap_bin], { 'on_stderr': funcref('s:error'), 'on_exit': funcref('s:exit'), 'rpc': v:true })

	if s:ch == 0
		echoerr 'server could not be started'
		return v:false
	elseif s:ch == -1
		echoerr 'server is not executable'
		return v:false
	endif

	return v:true
endfunction

function! s:sync(timer) abort
	if s:ch < 1
		echoerr 'server is not started'
		call s:stop()
		return
	endif

	call CocAction('fillDiagnostics', bufnr('%'))

	let l:diags = getloclist(win_getid())
	let l:changes = GitGutterGetHunks()

	call rpcnotify(s:ch, 'sync', l:diags, l:changes)

	let s:timer = timer_start(g:picomap_sync_interval, funcref('s:sync'), {})
endfunction

function! s:stop()
	if s:timer > 0
		call timer_stop(s:timer)
		let s:timer = 0
	endif
endfunction

function! picomap#show() abort
	let l:success = s:start()

	call rpcnotify(s:ch, 'show')

	if l:success
		let s:timer = timer_start(g:picomap_sync_interval, funcref('s:sync'), {})
	endif
endfunction

function! picomap#resize()
	if s:ch > 0
		call rpcnotify(s:ch, 'resize')
	endif
endfunction

function! picomap#hide()
	call s:stop()
	if s:ch > 0
		call rpcnotify(s:ch, 'close')
	endif
endfunction

function! picomap#debug() abort
	if s:debug_bufnr == 0
		let s:debug_bufnr = bufadd('miromap-debug')
		call setbufvar(s:debug_bufnr, '&swapfile', 0)
		call setbufvar(s:debug_bufnr, '&buftype', 'nofile')
		call setbufvar(s:debug_bufnr, '&undolevels', -1)
	endif
	execute ':buffer ' . s:debug_bufnr
endfunction

function! picomap#restart() abort
	call picomap#hide()

	if s:ch > 0
		call jobstop(s:ch)
		let s:ch = 0
	endif

	call picomap#show()
endfunction
