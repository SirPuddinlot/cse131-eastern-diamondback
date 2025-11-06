section .text
global our_code_starts_here
extern snek_error
extern _snek_print

fun_twoparams:
  push rbp
  mov rbp, rsp
  mov rax, 200
  mov [rbp - 8], rax
  mov rax, [rbp + 16]
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
  mov rax, 12
  push rax
  call fun_twoparams
  add rsp, 8
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
