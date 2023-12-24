A small, self-designed and interpreted assembler language. The interpreter is written in Rust

The language looks like this:

```nasm
mov rax 5
mov rbx 5
mov sp[0] rax
mov sp[1] 13
add sp[2] sp[0] sp[1]
add rcx rax rbx
call sp[3] return_9_minus_10
call print_10
sub rcx rax rbx
ret sp[3]

print_10:
    mov rax "Printing 10: "
    mov rbx "gu mo"
    add rcx rax rbx
    mov rax "{}"
    mov rbx rcx
    syscall printf
    leave

return_9_minus_10:
    mov rax 9
    mov rbx 10
    sub rcx rax rbx
    ret rcx
```