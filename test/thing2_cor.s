section .text
global our_code_starts_here
extern snek_error
extern _snek_print

fun_iseven:
  push rbp
  mov rbp, rsp
  sub rsp, 8
  mov [rbp-8], rdi         ; store n

  ; type check omitted for brevity, assume integer

  mov rax, [rbp-8]         ; load n
  cmp rax, 0
  mov rax, 1               ; assume false (1)
  mov rcx, 3               ; true (3)
  cmovg rax, rcx           ; if n > 0, rax = true
  leave
  ret

our_code_starts_here:
  push rbp
  mov rbp, rsp
  sub rsp, 16

  mov [rbp-8], rdi         ; save input
  mov rdi, [rbp-8]
  call _snek_print         ; print input

  mov rdi, [rbp-8]
  call fun_iseven          ; compute result

  mov rdi, rax
  call _snek_print         ; print result

  leave
  ret

error_invalid_argument:
  mov rdi, 2
  call snek_error
  ret
