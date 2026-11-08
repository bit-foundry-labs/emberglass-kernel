CC ?= gcc
LD ?= ld
CFLAGS = -std=gnu11 -O2 -Wall -Wextra -Werror -ffreestanding -fno-stack-protector -fno-stack-check -fno-pic -fno-pie -fno-builtin -fno-omit-frame-pointer -fno-asynchronous-unwind-tables -fno-unwind-tables -m64 -mno-red-zone -mno-mmx -mno-sse -mno-sse2 -mno-80387 -mcmodel=kernel -Iinclude
SOURCES = kernel/entry.c kernel/serial.c kernel/display.c kernel/memory.c kernel/memops.c
OBJECTS = $(patsubst kernel/%.c,build/%.o,$(SOURCES))
.PHONY: all check test clean inspect iso run
all: build/emberglass.elf
build:
	mkdir -p build
build/%.o: kernel/%.c | build
	$(CC) $(CFLAGS) -c $< -o $@
build/emberglass.elf: $(OBJECTS) linker.ld
	$(LD) -m elf_x86_64 -nostdlib -static -z max-page-size=0x1000 -z noexecstack -T linker.ld $(OBJECTS) -o $@
inspect: all
	readelf -h -l build/emberglass.elf
check: all
	python3 scripts/check_elf.py build/emberglass.elf
test: | build
	$(CC) -std=c11 -Wall -Wextra -Werror -fsanitize=address,undefined -fno-omit-frame-pointer -Iinclude tests/test_memory.c kernel/memory.c -o build/test_memory
	./build/test_memory
iso: check
	bash scripts/make_iso.sh
run: iso
	qemu-system-x86_64 -cpu max -m 256M -cdrom build/emberglass.iso -serial stdio -no-reboot -no-shutdown
clean:
	rm -rf build
