section .text
global our_code_starts_here
extern snek_error
extern _snek_print

fun_iseven:
  push rbp
  mov rbp, rsp
  sub rsp, 16
  mov rax, 4
  mov rsp, rbp
  pop rbp
  ret
our_code_starts_here:
  push rbp
  mov rbp, rsp
  mov [r15 + 0], rdi
  sub rsp, 16
  mov rax, [r15 + 0]
  push rax
  call fun_iseven
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
