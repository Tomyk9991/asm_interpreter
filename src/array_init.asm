mov rax 10 ; number of elements in the array
lea rbx sp[0]
init_loop:
    mov [rbx] 0
    add rbx rbx 1
    mov [rbx] 0
    add rbx rbx 1
    mov [rbx] 0
;    cmp rcx rax rbx
;    je rcx init_loop
ret 0