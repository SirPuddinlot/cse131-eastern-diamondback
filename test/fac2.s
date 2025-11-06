section .text
global our_code_starts_here
extern snek_error
extern _snek_print

fun_fact:
  push rbp
  mov rbp, rsp
  sub rsp, 48
  mov [rbp-8], rdi
  mov rax, 2
  mov [rbp - 16], rax
  mov rax, 2
  mov [rbp - 24], rax
loop_start_1:
  mov rax, [rbp - 16]
  mov [rbp - 32], rax
  mov rax, [rbp - 8]
  mov rcx, rax
  or rcx, [rbp - 32]
  test rcx, 1
  jne error_invalid_argument
  mov rcx, rax
  mov rax, [rbp - 32]
  cmp rax, rcx
  mov rax, 3
  mov rcx, 1
  cmovg rax, rcx
  cmp rax, 3
  je else_3
  mov rax, [rbp - 24]
  jmp loop_end_2
  jmp endif_4
else_3:
  mov rax, [rbp - 24]
  mov [rbp - 32], rax
  mov rax, [rbp - 16]
  mov rcx, rax
  or rcx, [rbp - 32]
  test rcx, 1
  jne error_invalid_argument
  sar rax, 1
  mov rcx, [rbp - 32]
  imul rax, rcx
  jo error_overflow
  mov [rbp - 24], rax
  mov rax, [rbp - 16]
  mov [rbp - 32], rax
  mov rax, 2
  mov rcx, rax
  or rcx, [rbp - 32]
  test rcx, 1
  jne error_invalid_argument
  add rax, [rbp - 32]
  jo error_overflow
  mov [rbp - 16], rax
endif_4:
  jmp loop_start_1
loop_end_2:
  mov rsp, rbp
  pop rbp
  ret
our_code_starts_here:
  push rbp
  mov rbp, rsp
  mov [r15 + 0], rdi
  sub rsp, 16
  mov rax, [r15 + 0]
  mov [rbp - 8], rax
  mov rdi, [rbp - 8]
  call fun_fact
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
