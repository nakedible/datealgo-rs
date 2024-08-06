datealgo::asm::secs_to_datetime:
 movabs  rax, 46387767571200
 lea     rcx, [rsi, +, rax]
 movabs  rdx, 46381619174399
 cmp     rsi, rdx
 cmovg   rcx, rax
 movabs  rdx, -4454547087429121353
 mov     rax, rcx
 mul     rdx
 shr     rdx, 16
 lea     esi, [4*rdx, +, 3]
 imul    rax, rsi, 963315389
 shr     rax, 47
 imul    r8d, eax, 146097
 sub     esi, r8d
 or      esi, 3
 imul    rsi, rsi, 2939745
 mov     r8d, esi
 imul    r8, r8, 1531969483
 shr     r8, 54
 imul    r8d, r8d, 2141
 add     r8d, 197913
 movzx   r9d, r8w
 shr     r8d, 16
 lea     r10d, [r8, -, 12]
 imul    eax, eax, 100
 mov     r11, rsi
 shr     r11, 32
 add     r11d, eax
 cmp     esi, -696719416
 sbb     r11d, -1
 cmp     esi, -696719416
 movzx   eax, r10b
 cmovb   eax, r8d
 imul    rdx, rdx, 86400
 sub     rcx, rdx
 imul    rcx, rcx, 71582789
 mov     edx, ecx
 shr     rcx, 32
 mov     esi, 4026531799
 imul    rdx, rsi
 shr     rdx, 58
 imul    rcx, rcx, 71582789
 mov     r8d, ecx
 imul    r8, rsi
 shr     r8, 58
 shr     rcx, 32
 add     r11d, -1468000
 imul    esi, r9d, 31345
 shr     esi, 26
 inc     sil
 mov     dword, ptr, [rdi], r11d
 mov     byte, ptr, [rdi, +, 4], al
 mov     byte, ptr, [rdi, +, 5], sil
 mov     byte, ptr, [rdi, +, 6], cl
 mov     byte, ptr, [rdi, +, 7], r8b
 mov     byte, ptr, [rdi, +, 8], dl
 mov     rax, rdi
 ret
