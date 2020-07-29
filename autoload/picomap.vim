let s:winid = 0
let s:ch = 0
let s:bufnr = 0
let s:timer = 0
let s:debug_bufnr = 0

let s:dir = expand('<sfile>:p:h')

let s:winopts = {
	\ 'relative': 'editor',
	\ 'anchor': 'NE',
	\ 'width': 2,
	\ 'focusable': v:false,
	\ 'style': 'minimal'
	\ }

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
		echoerr 'server already started'
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

	let l:lines = nvim_buf_get_lines(bufnr('%'), 0, -1, v:false)
	let l:diags = getloclist(win_getid())
	let l:changes = GitGutterGetHunks()
	let l:height = winheight(s:winid)
	let l:pos = getpos('.')
	let l:scroll = line('w0')

	call rpcnotify(s:ch, 'sync', s:bufnr, l:height, l:scroll, l:pos, l:lines, l:diags, l:changes)

	let s:timer = timer_start(g:picomap_sync_interval, funcref('s:sync'), {})
endfunction

function! s:stop()
	if s:timer > 0
		call timer_stop(s:timer)
		let s:timer = 0
	endif
	if s:ch > 0
		call jobstop(s:ch)
		let s:ch = 0
	endif
endfunction

function! picomap#show() abort
	let l:orig_winid = win_getid()

	let l:wininfo = getwininfo(l:orig_winid)[0]

	let s:bufnr = nvim_create_buf(v:false, v:true)

	let s:winopts.height = l:wininfo.height
	let s:winopts.col = l:wininfo.wincol + l:wininfo.width
	let s:winopts.row = l:wininfo.winrow - 1

	let s:winid = nvim_open_win(s:bufnr, v:true, s:winopts)

	setlocal filetype=picomap

	call nvim_win_set_option(s:winid, 'winhl', 'Normal:Picomap')
	call nvim_win_set_option(s:winid, 'winblend', g:picomap_winblend)

	call win_gotoid(l:orig_winid)

	let l:success = s:start()

	if l:success
		let s:timer = timer_start(g:picomap_sync_interval, funcref('s:sync'), {})
	endif
endfunction

function! picomap#update()
	if s:winid == 0
		return
	endif

	let l:wininfo = getwininfo(win_getid())[-1]

	let s:winopts.height = l:wininfo.height
	let s:winopts.col = l:wininfo.wincol + l:wininfo.width
	let s:winopts.row = l:wininfo.winrow - 1

	call nvim_win_set_config(s:winid, s:winopts)
endfunction

function! picomap#hide()
	call s:stop()
	if s:winid > 0
		call nvim_win_close(s:winid, v:false)
		let s:winid = 0
	endif
endfunction

function! picomap#click()
	if s:ch == 0
		return
	endif

	echo "clicked!"

	call rpcnotify(s:ch, 'click')
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
