datealgo::asm::systemtime_to_datetime:
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
	je .LBB19_1
	mov rax, qword ptr [rsp + 8]
	mov ecx, dword ptr [rsp + 16]
	mov qword ptr [rsp + 40], rax
	mov dword ptr [rsp + 48], ecx
	lea rdi, [rsp + 40]
	call qword ptr [rip + std::time::SystemTimeError::duration@GOTPCREL]
	cmp edx, 1
	sbb rax, -1
	cmp rax, r14
	ja .LBB19_6
	mov esi, 1000000000
	sub esi, edx
	test edx, edx
	cmove esi, edx
	neg rax
	jmp .LBB19_5
.LBB19_1:
	mov rax, qword ptr [rsp + 8]
	movabs rcx, 46381619174399
	cmp rax, rcx
	jbe .LBB19_2
.LBB19_6:
	xor eax, eax
	jmp .LBB19_7
.LBB19_2:
	mov esi, dword ptr [rsp + 16]
.LBB19_5:
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
	imul rcx, rcx, 71582789
	mov edx, ecx
	shr rcx, 32
	mov edi, 4026531799
	imul rdx, rdi
	shr rdx, 58
	imul rcx, rcx, 71582789
	mov r8d, ecx
	imul r8, rdi
	shr r8, 58
	shr rcx, 32
	add r11d, -1468000
	imul edi, r9d, 31345
	shr edi, 26
	inc dil
	mov dword ptr [rbx + 4], r11d
	mov byte ptr [rbx + 8], al
	mov byte ptr [rbx + 9], dil
	mov byte ptr [rbx + 10], cl
	mov byte ptr [rbx + 11], r8b
	mov byte ptr [rbx + 12], dl
	mov dword ptr [rbx + 16], esi
	mov eax, 1
.LBB19_7:
	mov dword ptr [rbx], eax
	mov rax, rbx
	add rsp, 56
	pop rbx
	pop r14
	ret
