mov rax ""
mov rbx 0
call rbx join_sub_string

; print the result
mov rax "{}"
syscall printf

ret 0


add_comma:
    add rcx rcx ", "
    leave

join_sub_string:
    add rcx "" rbx
    add rbx rbx 1
    cmp sp[0] rbx 10
    jne sp[0] add_comma
    add rax rax rcx
    cmp sp[0] rbx 10
    jne sp[0] join_sub_string
    ret rax