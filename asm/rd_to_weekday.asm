datealgo::asm::rd_to_weekday:
	add edi, 536895152
	movsxd rax, edi
	movabs rcx, 2635249153387078802
	imul rax, rcx
	add rax, rcx
	shr rax, 61
	ret
