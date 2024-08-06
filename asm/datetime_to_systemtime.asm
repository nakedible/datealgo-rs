datealgo::asm::datetime_to_systemtime:
	movsx ecx, byte ptr [rdi + 4]
	lea esi, [rcx + 12]
	xor r8d, r8d
	cmp ecx, 3
	setl r8b
	movsx eax, byte ptr [rdi + 5]
	mov r9d, dword ptr [rdi]
	mov edx, dword ptr [rdi + 12]
	cmovge esi, ecx
	sub r9d, r8d
	add r9d, 1468000
	imul rcx, r9, 1374389535
	mov r8, rcx
	shr r8, 37
	imul r9d, r9d, 1461
	shr r9d, 2
	shr rcx, 39
	imul esi, esi, 979
	add esi, -2919
	shr esi, 5
	sub eax, r8d
	add ecx, r9d
	add ecx, eax
	lea eax, [rsi + rcx]
	add eax, -307
	cmp eax, 1073719447
	ja .LBB20_1
	lea eax, [rsi + rcx]
	add eax, -536895459
	movsx rcx, byte ptr [rdi + 8]
	movsx rsi, byte ptr [rdi + 7]
	movsx rdi, byte ptr [rdi + 6]
	cdqe
	imul rax, rax, 86400
	imul rdi, rdi, 3600
	imul rsi, rsi, 60
	add rsi, rcx
	add rsi, rax
	add rsi, rdi
	jns .LBB20_2
	test edx, edx
	je .LBB20_10
	not rsi
	mov eax, 1000000000
	sub eax, edx
	cmp edx, 1000000001
	jb .LBB20_9
	mov ecx, eax
	shr ecx, 9
	imul rcx, rcx, 281475
	shr rcx, 39
	add rsi, rcx
	imul ecx, ecx, 1000000000
	sub eax, ecx
.LBB20_9:
	lea rdi, [rip + .L__unnamed_1]
	mov edx, eax
	jmp qword ptr [rip + std::time::SystemTime::checked_sub@GOTPCREL]
.LBB20_1:
	xor esi, esi
.LBB20_2:
	cmp edx, 1000000000
	jb .LBB20_4
	mov eax, edx
	shr eax, 9
	imul rax, rax, 281475
	shr rax, 39
	add rsi, rax
	imul eax, eax, 1000000000
	sub edx, eax
.LBB20_4:
	lea rdi, [rip + .L__unnamed_1]
	jmp qword ptr [rip + std::time::SystemTime::checked_add@GOTPCREL]
.LBB20_10:
	neg rsi
	lea rdi, [rip + .L__unnamed_1]
	xor edx, edx
	jmp qword ptr [rip + std::time::SystemTime::checked_sub@GOTPCREL]
