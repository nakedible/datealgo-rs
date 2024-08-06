datealgo::asm::rd_to_isoweekdate:
 lea     eax, [rdi, +, 536895152]
 movsxd  rcx, eax
 movabs  rax, 2635249153387078802
 imul    rcx, rax
 add     rcx, rax
 shr     rcx, 61
 sub     edi, ecx
 lea     eax, [4*rdi, -, 2147385445]
 imul    rdx, rax, 963315389
 shr     rdx, 47
 imul    esi, edx, 146097
 sub     eax, esi
 or      eax, 3
 imul    rax, rax, 2939745
 imul    edx, edx, 100
 mov     rsi, rax
 shr     rsi, 32
 add     esi, edx
 cmp     eax, -696719416
 sbb     esi, -1
 lea     edx, [rsi, -, 1468000]
 dec     esi
 imul    rax, rsi, 1374389535
 mov     r8, rax
 shr     r8, 37
 imul    esi, esi, 1461
 shr     esi, 2
 shr     rax, 39
 add     r8d, edi
 add     eax, esi
 sub     r8d, eax
 mov     eax, r8d
 add     eax, 536895156
 cdqe
 imul    rax, rax, -1840700269
 shr     rax, 32
 add     eax, r8d
 add     eax, 536895156
 mov     esi, eax
 shr     esi, 2
 shr     eax, 31
 shl     rcx, 40
 add     eax, esi
 inc     eax
 movzx   eax, al
 shl     rax, 32
 or      rax, rcx
 or      rax, rdx
 ret
