section .text
global our_code_starts_here
extern snek_error
extern _snek_print

fun_isodd:
  push rbp
  mov rbp, rsp
  sub rsp, 32
  mov [rbp-8], rdi
  mov rax, [rbp - 8]
  mov [rbp - 16], rax
  mov rax, 0
  mov rcx, rax
  mov rax, [rbp - 16]
  cmp rax, rcx
  mov rax, 1
  mov rcx, 3
  cmovl rax, rcx
  cmp rax, 1
  je else_1
  mov rax, 0
  mov [rbp - 24], rax
  mov rax, [rbp - 8]
  mov rcx, rax
  or rcx, [rbp - 24]
  test rcx, 1
  jne error_invalid_argument
  mov rcx, rax
  mov rax, [rbp - 24]
  sub rax, rcx
  jo error_overflow
  mov [rbp - 16], rax
  mov rdi, [rbp - 16]
  call fun_isodd
  jmp endif_2
else_1:
  mov rax, [rbp - 8]
  mov [rbp - 16], rax
  mov rax, 0
  mov rcx, [rbp - 16]
  cmp rax, rcx
  mov rax, 3
  mov rcx, 1
  cmovne rax, rcx
  cmp rax, 1
  je else_3
  mov rax, 1
  jmp endif_4
else_3:
  mov rax, [rbp - 8]
  test rax, 1
  jne error_invalid_argument
  sub rax, 2
  mov [rbp - 16], rax
  mov rdi, [rbp - 16]
  call fun_iseven
endif_4:
endif_2:
  mov rsp, rbp
  pop rbp
  ret
fun_iseven:
  push rbp
  mov rbp, rsp
  sub rsp, 32
  mov [rbp-8], rdi
  mov rax, [rbp - 8]
  mov [rbp - 16], rax
  mov rax, 0
  mov rcx, [rbp - 16]
  cmp rax, rcx
  mov rax, 3
  mov rcx, 1
  cmovne rax, rcx
  cmp rax, 1
  je else_5
  mov rax, 3
  jmp endif_6
else_5:
  mov rax, [rbp - 8]
  test rax, 1
  jne error_invalid_argument
  sub rax, 2
  mov [rbp - 16], rax
  mov rdi, [rbp - 16]
  call fun_isodd
endif_6:
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
