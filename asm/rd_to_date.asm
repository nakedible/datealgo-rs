datealgo::asm::rd_to_date:
 lea     eax, [4*rdi, -, 2147385461]
 imul    rcx, rax, 963315389
 shr     rcx, 47
 imul    edx, ecx, 146097
 sub     eax, edx
 or      eax, 3
 imul    rax, rax, 2939745
 mov     edx, eax
 imul    rdx, rdx, 1531969483
 shr     rdx, 54
 imul    edx, edx, 2141
 add     edx, 197913
 mov     esi, edx
 shr     esi, 16
 lea     edi, [rsi, +, 244]
 imul    r8d, ecx, 100
 mov     rcx, rax
 shr     rcx, 32
 add     ecx, r8d
 cmp     eax, -696719416
 cmovb   edi, esi
 sbb     ecx, -1
 movzx   eax, dx
 add     ecx, -1468000
 imul    eax, eax, 31345
 shr     eax, 26
 shl     rax, 40
 movzx   edx, dil
 shl     rdx, 32
 or      rdx, rax
 or      rcx, rdx
 movabs  rax, 1099511627776
 add     rax, rcx
 ret
