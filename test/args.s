section .text
global our_code_starts_here
extern snek_error
extern _snek_print

fun_isodd:
  push rbp
  mov rbp, rsp
  sub rsp, 48
  mov rax, [rbp + 16]
  mov [rbp - 8], rax
  mov rax, [rbp + 24]
  mov [rbp - 16], rax
  mov rax, [rbp + 32]
  mov [rbp - 24], rax
  mov rax, [rbp + 40]
  mov [rbp - 32], rax
  mov rax, [rbp + 64]
  mov rcx, rax
  or rcx, [rbp - 32]
  test rcx, 1
  jne error_invalid_argument
  add rax, [rbp - 32]
  jo error_overflow
  mov rcx, rax
  or rcx, [rbp - 24]
  test rcx, 1
  jne error_invalid_argument
  add rax, [rbp - 24]
  jo error_overflow
  mov rcx, rax
  or rcx, [rbp - 16]
  test rcx, 1
  jne error_invalid_argument
  add rax, [rbp - 16]
  jo error_overflow
  mov rcx, rax
  or rcx, [rbp - 8]
  test rcx, 1
  jne error_invalid_argument
  add rax, [rbp - 8]
  jo error_overflow
  mov rsp, rbp
  pop rbp
  ret
our_code_starts_here:
  push rbp
  mov rbp, rsp
  mov [r15 + 0], rdi
  sub rsp, 16
  sub rsp, 8
  mov rax, 16
  push rax
  mov rax, 14
  push rax
  mov rax, 12
  push rax
  mov rax, 10
  push rax
  mov rax, 8
  push rax
  mov rax, 6
  push rax
  mov rax, 4
  push rax
  mov rax, 2
  push rax
  call fun_isodd
  add rsp, 64
  add rsp, 8
  push rax
  mov rdi, rax
  call _snek_print
  pop rax
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
