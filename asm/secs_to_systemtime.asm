datealgo::asm::secs_to_systemtime:
	mov eax, esi
	mov rsi, rdi
	test rdi, rdi
	js .LBB18_1
	mov ecx, eax
	shr ecx, 9
	imul rcx, rcx, 281475
	shr rcx, 39
	add rsi, rcx
	imul ecx, ecx, 1000000000
	sub eax, ecx
	lea rdi, [rip + .L__unnamed_1]
	mov edx, eax
	jmp qword ptr [rip + std::time::SystemTime::checked_add@GOTPCREL]
.LBB18_1:
	test eax, eax
	je .LBB18_4
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
	jmp qword ptr [rip + std::time::SystemTime::checked_sub@GOTPCREL]
.LBB18_4:
	neg rsi
	lea rdi, [rip + .L__unnamed_1]
	xor edx, edx
	jmp qword ptr [rip + std::time::SystemTime::checked_sub@GOTPCREL]
