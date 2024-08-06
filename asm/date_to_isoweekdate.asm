datealgo::asm::date_to_isoweekdate:
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
 lea     eax, [rcx, +, rsi]
 add     ecx, esi
 add     ecx, -307
 movsxd  rcx, ecx
 movabs  rdx, 2635249153387078802
 imul    rcx, rdx
 add     rcx, rdx
 shr     rcx, 61
 sub     eax, ecx
 lea     edx, [rax, -, 536895459]
 lea     edx, [4*rdx, -, 2147385445]
 imul    rsi, rdx, 963315389
 shr     rsi, 47
 imul    edi, esi, 146097
 sub     edx, edi
 or      edx, 3
 imul    rdx, rdx, 2939745
 imul    esi, esi, 100
 mov     rdi, rdx
 shr     rdi, 32
 add     edi, esi
 cmp     edx, -696719416
 sbb     edi, -1
 lea     edx, [rdi, -, 1468000]
 dec     edi
 imul    rsi, rdi, 1374389535
 mov     r8, rsi
 shr     r8, 37
 imul    edi, edi, 1461
 shr     edi, 2
 shr     rsi, 39
 add     eax, r8d
 add     eax, -536895459
 add     esi, edi
 sub     eax, esi
 mov     esi, eax
 add     esi, 536895156
 movsxd  rsi, esi
 imul    rsi, rsi, -1840700269
 shr     rsi, 32
 add     eax, esi
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
