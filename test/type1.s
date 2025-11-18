section .text
global our_code_starts_here
extern snek_error
extern _snek_print

our_code_starts_here:
  push rbp
  mov rbp, rsp
  mov [r15 + 0], rdi
  sub rsp, 16
  mov rax, [r15 + 0]
  mov [rbp - 8], rax
  mov rax, 6
  mov rcx, rax
  or rcx, [rbp - 8]
  test rcx, 1
  jne error_invalid_argument
  add rax, [rbp - 8]
  jo error_overflow
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

error_bad_cast:
  mov rdi, 3
  call snek_error
  ret
