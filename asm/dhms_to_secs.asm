datealgo::asm::dhms_to_secs:
 lea     eax, [rdi, +, 536895152]
 cmp     eax, 1073719447
 ja      .LBB7_1
 mov     rax, rdi
 shr     rax, 48
 movzx   eax, al
 mov     rcx, rdi
 shr     rcx, 40
 mov     rdx, rdi
 shr     rdx, 32
 movsxd  rsi, edi
 imul    rsi, rsi, 86400
 add     rsi, rax
 movzx   eax, dl
 imul    rdx, rax, 3600
 add     rdx, rsi
 movzx   eax, cl
 imul    rax, rax, 60
 add     rax, rdx
 ret
.LBB7_1:
 xor     eax, eax
 ret
