SROCS := $(wildcard *.s)
OBJS := $(SRCS:%.s=%.obj)
define link
	clang-cl -c $(addsuffix .s, $(1))
	lld-link $(addsuffix .obj, $(1)) /ENTRY:main /SUBSYSTEM:CONSOLE /MACHINE:X64
endef

define nasmlink
	nasm -fwin64 $(addsuffix .asm, $(1))
	lld-link $(addsuffix .obj, $(1)) /ENTRY:main /SUBSYSTEM:CONSOLE /MACHINE:X64

endef
test:
#	nasm -fwin64 $(addsuffix .asm, $@)
	lld-link $(addsuffix .obj, $@) /ENTRY:main /SUBSYSTEM:CONSOLE /MACHINE:X64	
#
#cpp1:
#	clang-cl cpp1.cpp -c
#test:
#	$(call link, test)
#test1:
#	$(call link, test1)
#test2:
#	$(call nasmlink, test2)
#test3:
#	nasm -fwin64 $(addsuffix .asm, $@)
#	lld-link $(addsuffix .obj, $@) /ENTRY:main /SUBSYSTEM:CONSOLE /MACHINE:X64 /DEFAULTLIB:kernel32.lib user32.lib
#test4:
#	nasm -fwin64 $(addsuffix .asm, $@)
#	lld-link $(addsuffix .obj, $@) /ENTRY:main /SUBSYSTEM:CONSOLE /MACHINE:X64 /DEFAULTLIB:kernel32.lib user32.lib
#test5:
#	nasm -fwin64 $(addsuffix .asm, $@)
#	lld-link $(addsuffix .obj, $@) /ENTRY:main /SUBSYSTEM:CONSOLE /MACHINE:X64	
#test6:
#	nasm -fwin64 $(addsuffix .asm, $@)
#	lld-link $(addsuffix .obj, $@) /ENTRY:main /SUBSYSTEM:CONSOLE /MACHINE:X64	
#
