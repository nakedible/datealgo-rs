datealgo::asm::secs_to_dhms:
 movabs  rax, 46387767571200
 lea     rcx, [rdi, +, rax]
 movabs  rdx, 46381619174399
 cmp     rdi, rdx
 cmovg   rcx, rax
 movabs  rdx, -4454547087429121353
 mov     rax, rcx
 mul     rdx
 shr     rdx, 16
 imul    rax, rdx, 86400
 sub     rcx, rax
 imul    rax, rcx, 71582789
 mov     ecx, eax
 shr     rax, 32
 mov     esi, 4026531799
 imul    rcx, rsi
 imul    rax, rax, 71582789
 movabs  rdi, 545460846592
 and     rdi, rax
 mov     r8d, eax
 imul    r8, rsi
 lea     eax, [rdx, -, 536895458]
 shr     rcx, 10
 movabs  rdx, 17732923532771328
 and     rdx, rcx
 shr     r8, 18
 movabs  rcx, 69269232549888
 and     rcx, r8
 or      rcx, rdx
 or      rcx, rdi
 or      rax, rcx
 ret
