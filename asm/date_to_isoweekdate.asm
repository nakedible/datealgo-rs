datealgo::asm::date_to_isoweekdate:
	mov rax, rdi
	shr rax, 32
	movsx ecx, al
	lea edx, [rcx + 12]
	xor esi, esi
	cmp al, 3
	setl sil
	cmovge edx, ecx
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
	lea eax, [rcx + rdx]
	add ecx, edx
	add ecx, -307
	movsxd rcx, ecx
	movabs rdx, 2635249153387078802
	imul rcx, rdx
	add rcx, rdx
	shr rcx, 61
	sub eax, ecx
	lea edx, [rax - 536895459]
	lea edx, [4*rdx - 2147385445]
	imul rsi, rdx, 963315389
	shr rsi, 47
	imul edi, esi, 146097
	sub edx, edi
	or edx, 3
	imul rdx, rdx, 2939745
	imul esi, esi, 100
	mov rdi, rdx
	shr rdi, 32
	add edi, esi
	cmp edx, -696719416
	sbb edi, -1
	lea edx, [rdi - 1468000]
	dec edi
	imul rsi, rdi, 1374389535
	mov r8, rsi
	shr r8, 37
	imul edi, edi, 1461
	shr edi, 2
	shr rsi, 39
	add eax, r8d
	add eax, -536895459
	add esi, edi
	sub eax, esi
	mov esi, eax
	add esi, 536895156
	movsxd rsi, esi
	imul rsi, rsi, -1840700269
	shr rsi, 32
	add eax, esi
	add eax, 536895156
	mov esi, eax
	shr esi, 2
	shr eax, 31
	shl rcx, 40
	add eax, esi
	inc eax
	movzx eax, al
	shl rax, 32
	or rax, rcx
	or rax, rdx
	ret
