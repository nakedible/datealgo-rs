datealgo::asm::datetime_to_secs:
	mov ecx, dword ptr [rdi]
	movsx eax, byte ptr [rdi + 4]
	lea edx, [rax + 12]
	xor esi, esi
	cmp eax, 3
	setl sil
	movsx r8d, byte ptr [rdi + 5]
	cmovge edx, eax
	sub ecx, esi
	add ecx, 1468000
	imul rax, rcx, 1374389535
	mov rsi, rax
	shr rsi, 37
	imul r9d, ecx, 1461
	shr r9d, 2
	shr rax, 39
	imul ecx, edx, 979
	add ecx, -2919
	shr ecx, 5
	sub r8d, esi
	add eax, r9d
	add eax, r8d
	lea edx, [rcx + rax]
	add edx, -307
	cmp edx, 1073719447
	ja .LBB9_1
	add eax, ecx
	add eax, -536895459
	movsx rcx, byte ptr [rdi + 8]
	movsx rdx, byte ptr [rdi + 7]
	movsx rsi, byte ptr [rdi + 6]
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
