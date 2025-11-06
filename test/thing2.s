section .text
global our_code_starts_here
extern snek_error
extern _snek_print

fun_iseven:
  push rbp
  mov rbp, rsp
  mov [rbp-8], rdi
  sub rsp, 32
  mov rax, [rbp - 8]
  mov [rbp - 16], rax
  mov rax, 0
  mov rcx, rax
  mov rax, [rbp - 16]
  cmp rax, rcx
  mov rax, 1
  mov rcx, 3
  cmovg rax, rcx
  cmp rax, 1
  je else_1
  mov rax, 3
  jmp endif_2
else_1:
  mov rax, 1
endif_2:
  mov rsp, rbp
  pop rbp
  ret
our_code_starts_here:
  push rbp
  mov rbp, rsp
  sub rsp, 16
  mov [rbp - 8], rdi
  mov rdi, rax
  call _snek_print
  mov [rbp - 16], rdi
  mov [rbp - 8], rax
  mov rdi, [rbp - 8]
  call fun_iseven
  mov rdi, rax
  call _snek_print
  mov rsp, rbp
  pop rbp
  ret

error_overflow:
  mov rdi, 1
  call snek_error
  ret

error_invalid_argument:
  mov rdi, 2
  call snek_error
  ret
