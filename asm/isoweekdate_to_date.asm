datealgo::asm::isoweekdate_to_date:
 mov     rax, rdi
 shr     rax, 32
 mov     rcx, rdi
 shr     rcx, 40
 movzx   ecx, cl
 add     edi, 1467999
 imul    rdx, rdi, 1374389535
 mov     rsi, rdx
 shr     rsi, 37
 imul    edi, edi, 1461
 shr     edi, 2
 shr     rdx, 39
 sub     edx, esi
 movzx   eax, al
 lea     esi, [8*rax]
 sub     esi, eax
 add     esi, ecx
 add     esi, edi
 add     esi, edx
 lea     eax, [rdx, +, rdi]
 add     eax, 4
 cdqe
 movabs  rcx, 2635249153387078802
 imul    rcx, rax
 shr     rcx, 61
 dec     ecx
 movzx   eax, cl
 sub     esi, eax
 lea     eax, [4*rsi, +, 1207]
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
