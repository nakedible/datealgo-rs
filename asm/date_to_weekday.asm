datealgo::asm::date_to_weekday:
	movabs rax, 281474976710655
	and rax, rdi
	mov rcx, rax
	shr rcx, 32
	cmp cl, 3
	sbb edi, 0
	movzx edx, cl
	lea esi, [rdx + 12]
	cmp cl, 3
	cmovae esi, edx
	shr rax, 40
	add edi, 1468000
	imul rcx, rdi, 1374389535
	mov rdx, rcx
	shr rdx, 37
	lea edi, [rdi + 4*rdi]
	shr edi, 2
	shr rcx, 39
	imul esi, esi, 979
	add esi, -2855
	shr esi, 5
	sub eax, edx
	add ecx, edi
	add ecx, eax
	add ecx, esi
	imul eax, ecx, 613566756
	shr eax, 29
	ret
