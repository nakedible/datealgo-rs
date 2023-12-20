datealgo::asm::datetime_to_secs:
	mov ecx, dword ptr [rdi]
	movzx eax, byte ptr [rdi + 4]
	cmp eax, 3
	sbb ecx, 0
	lea edx, [rax + 12]
	cmp eax, 3
	cmovae edx, eax
	movzx esi, byte ptr [rdi + 5]
	add ecx, 1468000
	imul rax, rcx, 1374389535
	mov r8, rax
	shr r8, 37
	imul r9d, ecx, 1461
	shr r9d, 2
	shr rax, 39
	imul ecx, edx, 979
	add ecx, -2919
	shr ecx, 5
	sub esi, r8d
	add eax, r9d
	add eax, esi
	lea edx, [rcx + rax]
	add edx, -307
	cmp edx, 1073719447
	ja .LBB9_1
	add eax, ecx
	add eax, -536895459
	movzx ecx, byte ptr [rdi + 8]
	movzx edx, byte ptr [rdi + 7]
	movzx esi, byte ptr [rdi + 6]
	cdqe
	imul rdi, rax, 86400
	imul rsi, rsi, 3600
	imul rax, rdx, 60
	add rax, rcx
	add rax, rsi
	add rax, rdi
	ret
.LBB9_1:
	xor eax, eax
	ret
