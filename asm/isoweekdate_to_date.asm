datealgo::asm::isoweekdate_to_date:
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
	movzx eax, al
	lea esi, [8*rax]
	sub esi, eax
	add esi, ecx
	add esi, edi
	add esi, edx
	lea eax, [rdx + rdi]
	add eax, 4
	cdqe
	movabs rcx, 2635249153387078802
	imul rcx, rax
	shr rcx, 61
	dec ecx
	movzx eax, cl
	sub esi, eax
	lea ecx, [4*rsi + 1207]
	imul rax, rcx, 963315389
	shr rax, 47
	imul edx, eax, 146097
	sub ecx, edx
	or ecx, 3
	imul rcx, rcx, 2939745
	mov edx, ecx
	imul rdx, rdx, 1531969483
	shr rdx, 54
	imul edx, edx, 2141
	add edx, 197913
	movzx esi, dx
	shr edx, 16
	lea edi, [rdx + 244]
	imul r8d, eax, 100
	mov rax, rcx
	shr rax, 32
	add eax, r8d
	cmp ecx, -696719416
	cmovb edi, edx
	sbb eax, -1
	add eax, -1468000
	imul ecx, esi, 31345
	shr ecx, 26
	inc ecx
	shl rcx, 40
	movzx edx, dil
	shl rdx, 32
	or rdx, rcx
	or rax, rdx
	ret
