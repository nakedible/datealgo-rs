datealgo::asm::rd_to_date:
	lea ecx, [4*rdi - 2147385461]
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
