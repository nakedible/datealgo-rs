datealgo::asm::next_date:
	movabs rcx, 1099511627776
	movabs rax, 281474976710655
	and rax, rdi
	mov rdx, rax
	shr rdx, 32
	mov rsi, rdi
	shr rsi, 40
	cmp sil, 28
	jge .LBB4_1
.LBB4_3:
	add rdi, rcx
	movabs rcx, 280375465082880
	and rcx, rdi
.LBB4_8:
	movzx edx, dl
	shl rdx, 32
	mov eax, eax
	or rax, rcx
	or rax, rdx
	ret
.LBB4_1:
	cmp dl, 2
	jne .LBB4_2
	imul r8d, edi, -1030792151
	add r8d, 85899345
	cmp r8d, 171798691
	mov r8d, 15
	mov r9d, 3
	cmovb r9d, r8d
	test r9d, edi
	sete r8b
	or r8b, 28
	cmp r8b, sil
	ja .LBB4_3
	jmp .LBB4_7
.LBB4_2:
	mov r8d, edx
	sar r8b, 3
	xor r8b, dl
	or r8b, 30
	cmp r8b, sil
	jg .LBB4_3
	cmp dl, 12
	jge .LBB4_5
.LBB4_7:
	inc rdx
	jmp .LBB4_8
.LBB4_5:
	inc rax
	mov edx, 1
	jmp .LBB4_8
