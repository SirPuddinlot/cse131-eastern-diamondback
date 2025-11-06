section .text
global our_code_starts_here
extern snek_error

our_code_starts_here:
  push rbp
  mov  rbp, rsp
  sub  rsp, 32                ; reserve space for 4 locals (8 bytes each)

  mov  rax, 2
  mov  [rbp - 8], rax         ; local var1 = 2
  mov  rax, 2
  mov  [rbp - 16], rax        ; local var2 = 2

loop_start_1:
  mov  rax, [rbp - 8]
  mov  [rbp - 24], rax        ; temp = x

  mov  rcx, rdi               ; rcx = input
  or   rcx, [rbp - 24]
  test rcx, 1
  jne  error_invalid_argument

  cmp  [rbp - 24], rax        ; compare temp and rax (same)
  mov  rax, 1
  mov  rcx, 3
  cmovg rax, rcx
  cmp  rax, 1
  je   else_3

  mov  rax, [rbp - 16]
  jmp  loop_end_2
  jmp  endif_4

else_3:
  mov  rax, [rbp - 16]
  mov  [rbp - 24], rax

  mov  rax, [rbp - 8]
  mov  rcx, rax
  or   rcx, [rbp - 24]
  test rcx, 1
  jne  error_invalid_argument

  sar  rax, 1
  imul rax, [rbp - 24]
  jo   error_overflow
  mov  [rbp - 16], rax

  mov  rax, [rbp - 8]
  mov  [rbp - 24], rax

  mov  rax, 2
  mov  rcx, rax
  or   rcx, [rbp - 24]
  test rcx, 1
  jne  error_invalid_argument

  add  rax, [rbp - 24]
  jo   error_overflow
  mov  [rbp - 8], rax

endif_4:
  jmp  loop_start_1

loop_end_2:
  mov  rsp, rbp
  pop  rbp
  ret

error_overflow:
  mov  rdi, 1
  call snek_error
  ret

error_invalid_argument:
  mov  rdi, 2
  call snek_error
  ret
