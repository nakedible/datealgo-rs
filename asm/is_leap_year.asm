datealgo::asm::is_leap_year:
	imul eax, edi, -1030792151
	add eax, 85899345
	cmp eax, 171798691
	mov eax, 15
	mov ecx, 3
	cmovb ecx, eax
	test ecx, edi
	sete al
	ret
