datealgo::asm::systemtime_to_secs:
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
	je .LBB17_1
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
	ja .LBB17_6
	mov ecx, 1000000000
	sub ecx, edx
	test edx, edx
	cmove ecx, edx
	neg rax
	jmp .LBB17_3
.LBB17_1:
	mov rax, qword ptr [rsp + 16]
	movabs rcx, 46381619174399
	cmp rax, rcx
	jbe .LBB17_2
.LBB17_6:
	mov qword ptr [rbx], 0
	mov rax, rbx
	add rsp, 64
	pop rbx
	ret
.LBB17_2:
	mov ecx, dword ptr [rsp + 24]
.LBB17_3:
	mov qword ptr [rbx + 8], rax
	mov dword ptr [rbx + 16], ecx
	mov qword ptr [rbx], 1
	mov rax, rbx
	add rsp, 64
	pop rbx
	ret
