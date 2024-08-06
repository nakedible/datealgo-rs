datealgo::asm::date_to_rd:
	mov rax, rdi
	shr rax, 32
	movsx ecx, al
	lea edx, [rcx + 12]
	xor esi, esi
	cmp al, 3
	cmovge edx, ecx
	setl sil
	mov eax, edi
	sub eax, esi
	add eax, 1468000
	imul rcx, rax, 1374389535
	mov rsi, rcx
	shr rsi, 37
	shr rdi, 16
	sar edi, 24
	imul eax, eax, 1461
	shr eax, 2
	shr rcx, 39
	imul edx, edx, 979
	add edx, -2919
	shr edx, 5
	sub edi, esi
	add ecx, eax
	add ecx, edi
	lea eax, [rdx + rcx]
	add eax, -536895459
	ret
