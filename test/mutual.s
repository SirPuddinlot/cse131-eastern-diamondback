section .text
global our_code_starts_here
extern snek_error
extern _snek_print

fun_isodd:
  push rbp
  mov rbp, rsp
  sub rsp, 16
  mov rax, [rbp + 16]
  mov [rbp - 8], rax
  mov rax, 0
  mov [rbp - 16], rax
  mov rcx, rax
  or rcx, [rbp - 8]
  test rcx, 1
  jne error_invalid_argument
  mov rcx, [rbp - 8]
  mov rax, [rbp - 16]
  cmp rcx, rax
  mov rax, 3
  mov rcx, 1
  cmovl rax, rcx
  cmp rax, 3
  je else_1
  mov rax, 0
  mov [rbp - 8], rax
  mov rax, [rbp + 16]
  mov rcx, rax
  or rcx, [rbp - 8]
  test rcx, 1
  jne error_invalid_argument
  mov rcx, rax
  mov rax, [rbp - 8]
  sub rax, rcx
  jo error_overflow
  push rax
  call fun_isodd
  add rsp, 8
  jmp endif_2
else_1:
  mov rax, [rbp + 16]
  mov [rbp - 8], rax
  mov rax, 0
  cmp rax, [rbp - 8]
  mov rax, 1
  mov rcx, 3
  cmovne rax, rcx
  cmp rax, 3
  je else_3
  mov rax, 3
  jmp endif_4
else_3:
  mov rax, [rbp + 16]
  test rax, 1
  jne error_invalid_argument
  sub rax, 2
  jo error_overflow
  push rax
  call fun_iseven
  add rsp, 8
endif_4:
endif_2:
  mov rsp, rbp
  pop rbp
  ret
fun_iseven:
  push rbp
  mov rbp, rsp
  sub rsp, 16
  mov rax, [rbp + 16]
  mov [rbp - 8], rax
  mov rax, 0
  cmp rax, [rbp - 8]
  mov rax, 1
  mov rcx, 3
  cmovne rax, rcx
  cmp rax, 3
  je else_5
  mov rax, 1
  jmp endif_6
else_5:
  mov rax, [rbp + 16]
  test rax, 1
  jne error_invalid_argument
  sub rax, 2
  jo error_overflow
  push rax
  call fun_isodd
  add rsp, 8
endif_6:
  mov rsp, rbp
  pop rbp
  ret
our_code_starts_here:
  push rbp
  mov rbp, rsp
  mov [r15 + 0], rdi
  mov rax, [r15 + 0]
  push rax
  mov rdi, rax
  call _snek_print
  pop rax
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

error_bad_cast:
  mov rdi, 3
  call snek_error
  ret
