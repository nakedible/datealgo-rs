datealgo::asm::isoweekdate_to_rd:
	mov rax, rdi
	shr rax, 32
	mov rcx, rdi
	shr rcx, 40
	movzx ecx, cl
	add edi, 1467999
	imul rdx, rdi, 1374389535
	mov rsi, rdx
	shr rsi, 37
	imul edi, edi, 1461
	shr edi, 2
	shr rdx, 39
	sub edx, esi
	movzx esi, al
	lea eax, [8*rsi]
	sub eax, esi
	add eax, ecx
	add eax, edi
	add eax, edx
	lea ecx, [rdx + rdi]
	add ecx, 4
	movsxd rcx, ecx
	movabs rdx, 2635249153387078802
	imul rdx, rcx
	shr rdx, 61
	dec edx
	movzx ecx, dl
	sub eax, ecx
	add eax, -536895157
	ret
