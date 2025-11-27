section .text
global our_code_starts_here
extern snek_error
extern snek_print

our_code_starts_here:
  push rbp
  mov rbp, rsp
  mov [r15 + 0], rdi
  mov rax, [r15 + 0]
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
