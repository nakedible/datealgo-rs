datealgo::asm::isoweeks_in_year:
	lea eax, [rdi + 1467999]
	imul rcx, rax, 1374389535
	mov rdx, rcx
	shr rdx, 37
	imul eax, eax, 1461
	shr eax, 2
	shr rcx, 39
	sub ecx, edx
	add eax, ecx
	inc eax
	cdqe
	movabs rcx, 2635249153387078802
	imul rcx, rax
	shr rcx, 61
	cmp rcx, 3
	je .LBB16_3
	mov al, 52
	cmp ecx, 4
	jne .LBB16_4
	mov al, 53
	ret
.LBB16_3:
	imul eax, edi, -1030792151
	add eax, 85899345
	cmp eax, 171798691
	mov eax, 15
	mov ecx, 3
	cmovb ecx, eax
	test ecx, edi
	sete al
	or al, 52
.LBB16_4:
	ret
