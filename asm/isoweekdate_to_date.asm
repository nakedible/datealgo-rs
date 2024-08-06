datealgo::asm::isoweekdate_to_date:
	lea ecx, [rdi + 1467999]
	imul rax, rcx, 1374389535
	mov rdx, rax
	shr rdx, 37
	imul ecx, ecx, 1461
	shr ecx, 2
	shr rax, 39
	sub eax, edx
	mov rdx, rdi
	shr rdx, 8
	sar edx, 24
	lea esi, [8*rdx]
	sub esi, edx
	shr rdi, 16
	sar edi, 24
	add edi, esi
	add edi, ecx
	add edi, eax
	add eax, ecx
	add eax, 4
	cdqe
	movabs rcx, 2635249153387078802
	imul rcx, rax
	shr rcx, 61
	sub edi, ecx
	lea eax, [4*rdi + 1211]
	imul rcx, rax, 963315389
	shr rcx, 47
	imul edx, ecx, 146097
	sub eax, edx
	or eax, 3
	imul rax, rax, 2939745
	mov edx, eax
	imul rdx, rdx, 1531969483
	shr rdx, 54
	imul edx, edx, 2141
	add edx, 197913
	mov esi, edx
	shr esi, 16
	lea edi, [rsi + 244]
	imul r8d, ecx, 100
	mov rcx, rax
	shr rcx, 32
	add ecx, r8d
	cmp eax, -696719416
	cmovb edi, esi
	sbb ecx, -1
	movzx eax, dx
	add ecx, -1468000
	imul eax, eax, 31345
	shr eax, 26
	shl rax, 40
	movzx edx, dil
	shl rdx, 32
	or rdx, rax
	or rcx, rdx
	movabs rax, 1099511627776
	add rax, rcx
	ret
