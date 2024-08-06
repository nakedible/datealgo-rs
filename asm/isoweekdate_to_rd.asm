datealgo::asm::isoweekdate_to_rd:
	lea eax, [rdi + 1467999]
	imul rcx, rax, 1374389535
	mov rdx, rcx
	shr rdx, 37
	imul esi, eax, 1461
	shr esi, 2
	shr rcx, 39
	sub ecx, edx
	mov rdx, rdi
	shr rdx, 8
	sar edx, 24
	lea eax, [8*rdx]
	sub eax, edx
	shr rdi, 16
	sar edi, 24
	add eax, edi
	add eax, esi
	add eax, ecx
	add ecx, esi
	add ecx, 4
	movsxd rcx, ecx
	movabs rdx, 2635249153387078802
	imul rdx, rcx
	shr rdx, 61
	sub eax, edx
	add eax, -536895156
	ret
