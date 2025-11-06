section .text
global our_code_starts_here
extern snek_error
extern _snek_print

our_code_starts_here:
  push rbp
  mov rbp, rsp
  mov [r15 + 0], rdi
  sub rsp, 32
  mov rax, 2
  mov [rbp - 8], rax
  mov rax, 2
  mov [rbp - 16], rax
loop_start_1:
  mov rax, [rbp - 8]
  mov [rbp - 24], rax
  mov rax, [r15 + 0]
  mov rcx, rax
  or rcx, [rbp - 24]
  test rcx, 1
  jne error_invalid_argument
  mov rcx, rax
  mov rax, [rbp - 24]
  cmp rax, rcx
  mov rax, 3
  mov rcx, 1
  cmovg rax, rcx
  cmp rax, 3
  je else_3
  mov rax, [rbp - 16]
  jmp loop_end_2
  jmp endif_4
else_3:
  mov rax, [rbp - 16]
  mov [rbp - 24], rax
  mov rax, [rbp - 8]
  mov rcx, rax
  or rcx, [rbp - 24]
  test rcx, 1
  jne error_invalid_argument
  sar rax, 1
  mov rcx, [rbp - 24]
  imul rax, rcx
  jo error_overflow
  mov [rbp - 16], rax
  mov rax, [rbp - 8]
  mov [rbp - 24], rax
  mov rax, 2
  mov rcx, rax
  or rcx, [rbp - 24]
  test rcx, 1
  jne error_invalid_argument
  add rax, [rbp - 24]
  jo error_overflow
  mov [rbp - 8], rax
endif_4:
  jmp loop_start_1
loop_end_2:
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
