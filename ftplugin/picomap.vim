if exists('b:picomap_loaded')
	finish
endif

let b:picomap_loaded = 1

setlocal bufhidden=hide
	\ buftype=nofile
	\ nowrap
	\ noswapfile
	\ undolevels=-1

let s:ctermbg_cursor = 237
let s:guibg_cursor = '#2a3158'

let s:ctermbg_view = 235
let s:guibg_view = '#1e2132'

let s:diag_ctermfg = [
	\ 234,
	\ 130,
	\ 125,
	\ ]

let s:diag_guifg = [
	\ '#161821',
	\ '#c57339',
	\ '#cc517a',
	\ ]

let s:change_ctermfg = [
	\ 234,
	\ 64,
	\ ]

let s:change_guifg = [
	\ '#161821',
	\ '#668e3d'
	\ ]

for i in range(len(s:change_ctermfg)) 
	let syntax = printf('picomap_change%02d', i)
	execute('highlight ' . syntax . ' ctermbg=NONE ctermfg=' . s:change_ctermfg[i] . ' guibg=NONE guifg=' . s:change_guifg[i])
	execute('highlight ' . syntax . 'cursor ctermbg=' . s:ctermbg_cursor . ' ctermfg=' . s:change_ctermfg[i] . ' guibg=' . s:guibg_cursor . ' guifg=' . s:change_guifg[i])
	execute('highlight ' . syntax . 'view ctermbg=' . s:ctermbg_view . ' ctermfg=' . s:change_ctermfg[i] . ' guibg=' . s:guibg_view . ' guifg=' . s:change_guifg[i])
	call matchadd(syntax, printf('\(▖\|▘\|▌\| \).%02d.. $', i))
	call matchadd(syntax . 'cursor', printf('\(▖\|▘\|▌\| \).%02d..c$', i))
	call matchadd(syntax . 'view', printf('\(▖\|▘\|▌\| \).%02d..v$', i))
endfor

for i in range(len(s:diag_ctermfg))
	let syntax = printf('picomap_diag%02d', i)
	execute('highlight ' . syntax . ' ctermbg=NONE ctermfg=' . s:diag_ctermfg[i] . ' guibg=NONE guifg=' . s:diag_guifg[i]) 
	execute('highlight ' . syntax . 'cursor ctermbg=' . s:ctermbg_cursor . ' ctermfg=' . s:diag_ctermfg[i] . ' guibg=' . s:guibg_cursor . ' guifg=' . s:diag_guifg[i]) 
	execute('highlight ' . syntax . 'view ctermbg=' . s:ctermbg_view . ' ctermfg=' . s:diag_ctermfg[i] . ' guibg=' . s:guibg_view . ' guifg=' . s:diag_guifg[i]) 
	call matchadd(syntax, printf('\(▖\|▘\|▌\| \)..%02d $', i))
	call matchadd(syntax . 'cursor', printf('\(▖\|▘\|▌\| \)..%02dc$', i))
	call matchadd(syntax . 'view', printf('\(▖\|▘\|▌\| \)..%02dv$', i))
endfor
