mov rax 10 ; number of elements in the array
lea rbx sp[0]
init_loop:
    mov [rbx] 0
    add rbx rbx 1
    cmp rcx rax rbx
    jne rcx init_loop

    lea rbx sp[0]
    mov rax 10
    ; Modifying elements of the array
    mov sp[10] 0
process_loop:
    mov [rbx] sp[10]
    add rbx rbx 1
    add sp[10] sp[10] 1
    cmp rcx rax rbx
    jne rcx process_loop
ret 0