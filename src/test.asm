mov rax "{}"
lea rbx sp[13]
mov [rbx] 13
syscall printf
leave
; loads 13 into whatever is written in rbx
; mov rbx = 5
; mov [rbx] 13 results in
; mov 0x5 13
