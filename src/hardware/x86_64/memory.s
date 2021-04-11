.text
.intel_syntax noprefix
.global get_cr0,set_cr0
.global set_cr3,get_cr3,get_cr4,set_cr4

set_cr0:
    mov cr0,rdi
    ret

get_cr0:
    mov rax,cr0
    ret

set_cr3:
    mov cr3,rdi
    ret

get_cr3:
    mov rax,cr3
    ret

set_cr4:
    mov cr4,rdi
    ret

get_cr4:
    mov rax,cr4
    ret