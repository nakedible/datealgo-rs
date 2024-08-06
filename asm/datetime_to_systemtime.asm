datealgo::asm::datetime_to_systemtime:
 movzx   eax, byte, ptr, [rdi, +, 4]
 cmp     eax, 3
 mov     ecx, dword, ptr, [rdi]
 mov     edx, dword, ptr, [rdi, +, 12]
 sbb     ecx, 0
 lea     esi, [rax, +, 12]
 cmp     eax, 3
 cmovae  esi, eax
 movzx   r8d, byte, ptr, [rdi, +, 5]
 add     ecx, 1468000
 imul    rax, rcx, 1374389535
 mov     r9, rax
 shr     r9, 37
 imul    r10d, ecx, 1461
 shr     r10d, 2
 shr     rax, 39
 imul    ecx, esi, 979
 add     ecx, -2919
 shr     ecx, 5
 sub     r8d, r9d
 add     eax, r10d
 add     eax, r8d
 lea     esi, [rcx, +, rax]
 add     esi, -307
 cmp     esi, 1073719447
 ja      .LBB20_1
 add     eax, ecx
 add     eax, -536895459
 movzx   ecx, byte, ptr, [rdi, +, 8]
 movzx   esi, byte, ptr, [rdi, +, 7]
 movzx   edi, byte, ptr, [rdi, +, 6]
 cdqe
 imul    rax, rax, 86400
 imul    rdi, rdi, 3600
 imul    rsi, rsi, 60
 add     rsi, rcx
 add     rsi, rax
 add     rsi, rdi
 jns     .LBB20_2
 test    edx, edx
 je      .LBB20_10
 not     rsi
 mov     eax, 1000000000
 sub     eax, edx
 cmp     edx, 1000000001
 jb      .LBB20_9
 mov     ecx, eax
 shr     ecx, 9
 imul    rcx, rcx, 281475
 shr     rcx, 39
 add     rsi, rcx
 imul    ecx, ecx, 1000000000
 sub     eax, ecx
.LBB20_9:
 lea     rdi, [rip, +, .L__unnamed_1]
 mov     edx, eax
 jmp     qword, ptr, [rip, +, _ZN3std4time10SystemTime11checked_sub17h9a81180a54e9a179E@GOTPCREL]
.LBB20_1:
 xor     esi, esi
.LBB20_2:
 cmp     edx, 1000000000
 jb      .LBB20_4
 mov     eax, edx
 shr     eax, 9
 imul    rax, rax, 281475
 shr     rax, 39
 add     rsi, rax
 imul    eax, eax, 1000000000
 sub     edx, eax
.LBB20_4:
 lea     rdi, [rip, +, .L__unnamed_1]
 jmp     qword, ptr, [rip, +, _ZN3std4time10SystemTime11checked_add17h8e270767f4ff8e12E@GOTPCREL]
.LBB20_10:
 neg     rsi
 lea     rdi, [rip, +, .L__unnamed_1]
 xor     edx, edx
 jmp     qword, ptr, [rip, +, _ZN3std4time10SystemTime11checked_sub17h9a81180a54e9a179E@GOTPCREL]
