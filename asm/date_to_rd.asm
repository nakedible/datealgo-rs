datealgo::asm::date_to_rd:
 movabs  rax, 281474976710655
 and     rax, rdi
 mov     rcx, rax
 shr     rcx, 32
 cmp     cl, 3
 sbb     edi, 0
 movzx   edx, cl
 lea     esi, [rdx, +, 12]
 cmp     cl, 3
 cmovae  esi, edx
 shr     rax, 40
 add     edi, 1468000
 imul    rcx, rdi, 1374389535
 mov     rdx, rcx
 shr     rdx, 37
 imul    edi, edi, 1461
 shr     edi, 2
 shr     rcx, 39
 imul    esi, esi, 979
 add     esi, -2919
 shr     esi, 5
 sub     eax, edx
 add     ecx, edi
 add     ecx, eax
 lea     eax, [rsi, +, rcx]
 add     eax, -536895459
 ret
