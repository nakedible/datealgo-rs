datealgo::asm::dhms_to_secs:
	lea eax, [rdi + 536895152]
	cmp eax, 1073719447
	ja .LBB7_1
	movsxd rax, edi
	imul rcx, rax, 86400
	mov rax, rdi
	shr rax, 32
	movsx rax, al
	imul rdx, rax, 3600
	mov rax, rdi
	shr rax, 40
	movsx rax, al
	imul rsi, rax, 60
	shr rdi, 48
	movsx rax, dil
	add rax, rcx
	add rax, rdx
	add rax, rsi
	ret
.LBB7_1:
	xor eax, eax
	ret
