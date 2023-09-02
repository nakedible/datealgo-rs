# Compilation provided by Compiler Explorer at https://godbolt.org/
example::rd_to_date:
  lea ecx, [4*rdi - 2147385461]
  imul rax, rcx, 963315389
  shr rax, 47
  imul edx, eax, 146097
  sub ecx, edx
  or ecx, 3
  imul rcx, rcx, 2939745
  mov edx, ecx
  imul rdx, rdx, 1531969483
  shr rdx, 54
  imul edx, edx, 2141
  add edx, 197913
  movzx esi, dx
  shr edx, 16
  lea edi, [rdx + 244]
  imul r8d, eax, 100
  mov rax, rcx
  shr rax, 32
  add eax, r8d
  cmp ecx, -696719416
  cmovb edi, edx
  sbb eax, -1
  add eax, -1468000
  imul ecx, esi, 31345
  shr ecx, 26
  inc ecx
  shl rcx, 40
  movzx edx, dil
  shl rdx, 32
  or rdx, rcx
  or rax, rdx
  ret

example::date_to_rd:
  mov rax, rdi
  shr rax, 32
  cmp al, 3
  mov ecx, edi
  sbb ecx, 0
  movzx edx, al
  lea esi, [rdx + 12]
  cmp al, 3
  cmovae esi, edx
  shr rdi, 40
  add ecx, 1468000
  movzx eax, dil
  imul rdx, rcx, 1374389535
  mov rdi, rdx
  shr rdi, 37
  imul ecx, ecx, 1461
  shr ecx, 2
  shr rdx, 39
  imul esi, esi, 979
  add esi, -2919
  shr esi, 5
  sub eax, edi
  add edx, ecx
  add edx, eax
  lea eax, [rsi + rdx]
  add eax, -536895459
  ret

example::rd_to_weekday:
  lea eax, [rdi + 536895460]
  imul rcx, rax, 613566757
  shr rcx, 32
  sub eax, ecx
  shr eax
  add eax, ecx
  shr eax, 2
  lea ecx, [8*rax]
  sub eax, ecx
  add eax, edi
  add eax, 536895460
  inc al
  ret

example::date_to_weekday:
  movabs rax, 281474976710655
  and rax, rdi
  mov rcx, rax
  shr rcx, 32
  cmp cl, 3
  sbb edi, 0
  movzx edx, cl
  lea esi, [rdx + 12]
  cmp cl, 3
  cmovae esi, edx
  shr rax, 40
  add edi, 1468000
  imul rcx, rdi, 1374389535
  mov rdx, rcx
  shr rdx, 37
  imul edi, edi, 1461
  shr edi, 2
  shr rcx, 39
  imul esi, esi, 979
  add esi, -2919
  shr esi, 5
  sub eax, edx
  add ecx, edi
  add ecx, eax
  add ecx, esi
  inc ecx
  imul rdx, rcx, 613566757
  shr rdx, 32
  mov eax, ecx
  sub eax, edx
  shr eax
  add eax, edx
  shr eax, 2
  lea edx, [8*rax]
  sub eax, edx
  add eax, ecx
  inc al
  ret

example::secs_to_dhms:
  movabs rax, 46387767571200
  lea rcx, [rdi + rax]
  movabs rdx, 46381619174399
  cmp rdi, rdx
  cmovg rcx, rax
  movabs rdx, -4454547087429121353
  mov rax, rcx
  mul rdx
  shr rdx, 16
  imul rax, rdx, 86400
  sub rcx, rax
  imul rsi, rcx, 71582789
  shr rsi, 32
  imul eax, esi, 60
  imul rdi, rcx, 1193047
  sub ecx, eax
  imul eax, esi, 2185
  shr eax, 17
  imul eax, eax, 60
  sub esi, eax
  lea eax, [rdx - 536895458]
  movzx edx, si
  shl rcx, 48
  shl rdx, 40
  or rdx, rcx
  movabs rcx, 270582939648
  and rcx, rdi
  or rcx, rdx
  or rax, rcx
  ret

example::dhms_to_secs:
  lea eax, [rdi + 536895152]
  cmp eax, 1073719447
  ja .LBB5_1
  mov rax, rdi
  shr rax, 48
  mov rcx, rdi
  shr rcx, 40
  mov rdx, rdi
  shr rdx, 32
  movsxd rsi, edi
  imul rsi, rsi, 86400
  movzx edx, dl
  imul rdx, rdx, 3600
  movzx ecx, cl
  imul rcx, rcx, 60
  movzx eax, al
  add rax, rsi
  add rax, rdx
  add rax, rcx
  ret
.LBB5_1:
  xor eax, eax
  ret

example::secs_to_datetime:
  movabs rax, 46387767571200
  lea rcx, [rsi + rax]
  movabs rdx, 46381619174399
  cmp rsi, rdx
  cmovg rcx, rax
  movabs rdx, -4454547087429121353
  mov rax, rcx
  mul rdx
  shr rdx, 16
  lea eax, [4*rdx + 3]
  imul r8, rax, 963315389
  shr r8, 47
  imul esi, r8d, 146097
  sub eax, esi
  or eax, 3
  imul r9, rax, 2939745
  mov eax, r9d
  imul rax, rax, 1531969483
  shr rax, 54
  imul esi, eax, 2141
  add esi, 197913
  mov r10d, esi
  shr r10d, 16
  lea r11d, [r10 - 12]
  imul r8d, r8d, 100
  mov rax, r9
  shr rax, 32
  add eax, r8d
  cmp r9d, -696719416
  sbb eax, -1
  cmp r9d, -696719416
  movzx r8d, r11b
  cmovb r8d, r10d
  movzx esi, si
  imul rdx, rdx, 86400
  sub rcx, rdx
  imul rdx, rcx, 71582789
  shr rdx, 32
  imul r9d, edx, 60
  imul r10, rcx, 1193047
  sub ecx, r9d
  imul r9d, edx, 2185
  shr r9d, 17
  imul r9d, r9d, 60
  sub edx, r9d
  movabs r9, -4294967296
  and r9, r10
  shr r10, 32
  movzx edx, dx
  shl rcx, 48
  shl rdx, 40
  or r9, rcx
  or r9, rdx
  mov rcx, r9
  shr rcx, 40
  shr r9, 48
  add eax, -1468000
  imul edx, esi, 31345
  shr edx, 26
  inc dl
  mov dword ptr [rdi], eax
  mov byte ptr [rdi + 4], r8b
  mov byte ptr [rdi + 5], dl
  mov byte ptr [rdi + 6], r10b
  mov byte ptr [rdi + 7], cl
  mov byte ptr [rdi + 8], r9b
  mov rax, rdi
  ret

example::datetime_to_secs:
  mov ecx, dword ptr [rdi]
  movzx eax, byte ptr [rdi + 4]
  cmp eax, 3
  sbb ecx, 0
  lea edx, [rax + 12]
  cmp eax, 3
  cmovae edx, eax
  movzx esi, byte ptr [rdi + 5]
  add ecx, 1468000
  imul rax, rcx, 1374389535
  mov r8, rax
  shr r8, 37
  imul r9d, ecx, 1461
  shr r9d, 2
  shr rax, 39
  imul ecx, edx, 979
  add ecx, -2919
  shr ecx, 5
  sub esi, r8d
  add eax, r9d
  add eax, esi
  lea edx, [rcx + rax]
  add edx, -307
  cmp edx, 1073719447
  ja .LBB7_1
  add eax, ecx
  add eax, -536895459
  movzx ecx, byte ptr [rdi + 8]
  movzx edx, byte ptr [rdi + 7]
  movzx esi, byte ptr [rdi + 6]
  cdqe
  imul rdi, rax, 86400
  imul rsi, rsi, 3600
  imul rax, rdx, 60
  add rax, rcx
  add rax, rsi
  add rax, rdi
  ret
.LBB7_1:
  xor eax, eax
  ret

example::is_leap_year:
  imul eax, edi, -1030792151
  add eax, 85899344
  ror eax, 2
  cmp eax, 42949673
  mov eax, 15
  mov ecx, 3
  cmovb ecx, eax
  test ecx, edi
  sete al
  ret

example::days_in_month:
  cmp sil, 2
  jne .LBB9_2
  imul eax, edi, -1030792151
  add eax, 85899344
  ror eax, 2
  cmp eax, 42949673
  mov eax, 15
  mov ecx, 3
  cmovb ecx, eax
  test ecx, edi
  sete al
  or al, 28
  ret
.LBB9_2:
  mov eax, esi
  shr al, 3
  xor al, sil
  or al, 30
  ret

example::systemtime_to_secs:
  push rbx
  sub rsp, 64
  mov rbx, rdi
  mov qword ptr [rsp + 32], rsi
  mov dword ptr [rsp + 40], edx
  lea rdi, [rsp + 8]
  lea rsi, [rsp + 32]
  xor edx, edx
  xor ecx, ecx
  call qword ptr [rip + std::time::SystemTime::duration_since@GOTPCREL]
  cmp qword ptr [rsp + 8], 0
  je .LBB10_1
  mov rax, qword ptr [rsp + 16]
  mov ecx, dword ptr [rsp + 24]
  mov qword ptr [rsp + 48], rax
  mov dword ptr [rsp + 56], ecx
  lea rdi, [rsp + 48]
  call qword ptr [rip + std::time::SystemTimeError::duration@GOTPCREL]
  cmp edx, 1
  sbb rax, -1
  movabs rcx, 46387741132800
  cmp rax, rcx
  ja .LBB10_6
  mov ecx, 1000000000
  sub ecx, edx
  test edx, edx
  cmove ecx, edx
  neg rax
  jmp .LBB10_3
.LBB10_1:
  mov rax, qword ptr [rsp + 16]
  movabs rcx, 46381619174399
  cmp rax, rcx
  jbe .LBB10_2
.LBB10_6:
  mov qword ptr [rbx], 0
  mov rax, rbx
  add rsp, 64
  pop rbx
  ret
.LBB10_2:
  mov ecx, dword ptr [rsp + 24]
.LBB10_3:
  mov qword ptr [rbx + 8], rax
  mov dword ptr [rbx + 16], ecx
  mov qword ptr [rbx], 1
  mov rax, rbx
  add rsp, 64
  pop rbx
  ret

example::secs_to_systemtime:
  mov eax, esi
  mov rsi, rdi
  test rdi, rdi
  js .LBB11_1
  mov ecx, eax
  shr ecx, 9
  imul rcx, rcx, 281475
  shr rcx, 39
  add rsi, rcx
  imul ecx, ecx, 1000000000
  sub eax, ecx
  lea rdi, [rip + .L__unnamed_1]
  mov edx, eax
  jmp qword ptr [rip + _ZN3std4time10SystemTime11checked_add17h6ac5fcb036f3f003E@GOTPCREL]
.LBB11_1:
  test eax, eax
  je .LBB11_4
  not rsi
  mov edx, 1000000000
  sub edx, eax
  mov eax, edx
  shr eax, 9
  imul rax, rax, 281475
  shr rax, 39
  add rsi, rax
  imul eax, eax, 1000000000
  sub edx, eax
  lea rdi, [rip + .L__unnamed_1]
  jmp qword ptr [rip + _ZN3std4time10SystemTime11checked_sub17h7e1caea028a19c77E@GOTPCREL]
.LBB11_4:
  neg rsi
  lea rdi, [rip + .L__unnamed_1]
  xor edx, edx
  jmp qword ptr [rip + _ZN3std4time10SystemTime11checked_sub17h7e1caea028a19c77E@GOTPCREL]

example::systemtime_to_datetime:
  push r14
  push rbx
  sub rsp, 56
  mov rbx, rdi
  movabs r14, 46387741132800
  mov qword ptr [rsp + 24], rsi
  mov dword ptr [rsp + 32], edx
  mov rdi, rsp
  lea rsi, [rsp + 24]
  xor edx, edx
  xor ecx, ecx
  call qword ptr [rip + std::time::SystemTime::duration_since@GOTPCREL]
  cmp qword ptr [rsp], 0
  je .LBB12_1
  mov rax, qword ptr [rsp + 8]
  mov ecx, dword ptr [rsp + 16]
  mov qword ptr [rsp + 40], rax
  mov dword ptr [rsp + 48], ecx
  lea rdi, [rsp + 40]
  call qword ptr [rip + std::time::SystemTimeError::duration@GOTPCREL]
  cmp edx, 1
  sbb rax, -1
  cmp rax, r14
  ja .LBB12_6
  mov esi, 1000000000
  sub esi, edx
  test edx, edx
  cmove esi, edx
  neg rax
  jmp .LBB12_5
.LBB12_1:
  mov rax, qword ptr [rsp + 8]
  movabs rcx, 46381619174399
  cmp rax, rcx
  jbe .LBB12_2
.LBB12_6:
  xor eax, eax
  jmp .LBB12_7
.LBB12_2:
  mov esi, dword ptr [rsp + 16]
.LBB12_5:
  lea rcx, [r14 + rax]
  add rcx, 26438400
  movabs rdx, -4454547087429121353
  mov rax, rcx
  mul rdx
  shr rdx, 16
  lea edi, [4*rdx + 3]
  imul rax, rdi, 963315389
  shr rax, 47
  imul r8d, eax, 146097
  sub edi, r8d
  or edi, 3
  imul rdi, rdi, 2939745
  mov r8d, edi
  imul r8, r8, 1531969483
  shr r8, 54
  imul r8d, r8d, 2141
  add r8d, 197913
  movzx r9d, r8w
  shr r8d, 16
  lea r10d, [r8 - 12]
  imul eax, eax, 100
  mov r11, rdi
  shr r11, 32
  add r11d, eax
  cmp edi, -696719416
  sbb r11d, -1
  cmp edi, -696719416
  movzx eax, r10b
  cmovb eax, r8d
  imul rdx, rdx, 86400
  sub rcx, rdx
  imul rdx, rcx, 71582789
  shr rdx, 32
  imul edi, edx, 60
  imul r8, rcx, 1193047
  sub ecx, edi
  imul edi, edx, 2185
  shr edi, 17
  imul edi, edi, 60
  sub edx, edi
  movabs rdi, -4294967296
  and rdi, r8
  shr r8, 32
  movzx edx, dx
  shl rcx, 48
  shl rdx, 40
  or rdi, rcx
  or rdi, rdx
  mov rcx, rdi
  shr rcx, 40
  shr rdi, 48
  add r11d, -1468000
  imul edx, r9d, 31345
  shr edx, 26
  inc dl
  mov dword ptr [rbx + 4], r11d
  mov byte ptr [rbx + 8], al
  mov byte ptr [rbx + 9], dl
  mov byte ptr [rbx + 10], r8b
  mov byte ptr [rbx + 11], cl
  mov byte ptr [rbx + 12], dil
  mov dword ptr [rbx + 16], esi
  mov eax, 1
.LBB12_7:
  mov dword ptr [rbx], eax
  mov rax, rbx
  add rsp, 56
  pop rbx
  pop r14
  ret

example::datetime_to_systemtime:
  movzx eax, byte ptr [rdi + 4]
  cmp eax, 3
  mov ecx, dword ptr [rdi]
  mov edx, dword ptr [rdi + 12]
  sbb ecx, 0
  lea esi, [rax + 12]
  cmp eax, 3
  cmovae esi, eax
  movzx r8d, byte ptr [rdi + 5]
  add ecx, 1468000
  imul rax, rcx, 1374389535
  mov r9, rax
  shr r9, 37
  imul r10d, ecx, 1461
  shr r10d, 2
  shr rax, 39
  imul ecx, esi, 979
  add ecx, -2919
  shr ecx, 5
  sub r8d, r9d
  add eax, r10d
  add eax, r8d
  lea esi, [rcx + rax]
  add esi, -307
  cmp esi, 1073719447
  ja .LBB13_1
  add eax, ecx
  add eax, -536895459
  movzx ecx, byte ptr [rdi + 8]
  movzx esi, byte ptr [rdi + 7]
  movzx edi, byte ptr [rdi + 6]
  cdqe
  imul rax, rax, 86400
  imul rdi, rdi, 3600
  imul rsi, rsi, 60
  add rsi, rcx
  add rsi, rax
  add rsi, rdi
  jns .LBB13_2
  test edx, edx
  je .LBB13_5
  not rsi
  mov eax, 1000000000
  sub eax, edx
  mov ecx, eax
  shr ecx, 9
  imul rcx, rcx, 281475
  shr rcx, 39
  add rsi, rcx
  imul ecx, ecx, 1000000000
  sub eax, ecx
  lea rdi, [rip + .L__unnamed_1]
  mov edx, eax
  jmp qword ptr [rip + _ZN3std4time10SystemTime11checked_sub17h7e1caea028a19c77E@GOTPCREL]
.LBB13_1:
  xor esi, esi
.LBB13_2:
  mov eax, edx
  shr eax, 9
  imul rax, rax, 281475
  shr rax, 39
  add rsi, rax
  imul eax, eax, 1000000000
  sub edx, eax
  lea rdi, [rip + .L__unnamed_1]
  jmp qword ptr [rip + _ZN3std4time10SystemTime11checked_add17h6ac5fcb036f3f003E@GOTPCREL]
.LBB13_5:
  neg rsi
  lea rdi, [rip + .L__unnamed_1]
  xor edx, edx
  jmp qword ptr [rip + _ZN3std4time10SystemTime11checked_sub17h7e1caea028a19c77E@GOTPCREL]

.L__unnamed_1:
  .zero 12
  .zero 4

