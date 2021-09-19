entry:
  mvi 10, a
  call add_4
  hlt
add_4:
  addi 4, a
  ret

; side by side mode from assembler:
;               | entry:
; 00:     7f 0a | 	mvi 10, a
; 02:     bc 05 | 	call add_4
; 04:        c7 | 	hlt
;               | add_4:
; 05:     0c 04 | 	addi 4, a
; 07:        bd | 	ret