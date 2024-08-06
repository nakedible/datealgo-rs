datealgo::asm::systemtime_to_secs:
	push rbx
	sub rsp, 48
	mov rbx, rdi
	mov qword ptr [rsp + 32], rsi
	mov dword ptr [rsp + 40], edx
	lea rdi, [rsp + 8]
	lea rsi, [rsp + 32]
	xor edx, edx
	xor ecx, ecx
	call qword ptr [rip + std::time::SystemTime::duration_since@GOTPCREL]
	cmp qword ptr [rsp + 8], 0
	mov rax, qword ptr [rsp + 16]
	je .LBB17_1
	mov edx, dword ptr [rsp + 24]
	cmp edx, 1
	sbb rax, -1
	movabs rcx, 46387741132800
	cmp rax, rcx
	ja .LBB17_5
	mov ecx, 1000000000
	sub ecx, edx
	test edx, edx
	cmove ecx, edx
	neg rax
	jmp .LBB17_3
.LBB17_1:
	movabs rcx, 46381619174399
	cmp rax, rcx
	jbe .LBB17_2
.LBB17_5:
	mov qword ptr [rbx], 0
	mov rax, rbx
	add rsp, 48
	pop rbx
	ret
.LBB17_2:
	mov ecx, dword ptr [rsp + 24]
.LBB17_3:
	mov qword ptr [rbx + 8], rax
	mov dword ptr [rbx + 16], ecx
	mov qword ptr [rbx], 1
	mov rax, rbx
	add rsp, 48
	pop rbx
	ret
