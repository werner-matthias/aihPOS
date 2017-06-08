SRCDIR=src
OBJDIR=obj
BUILDDIR=build
OBJCOPY=arm-none-eabi-objcopy
OBJDUMP=arm-none-eabi-objdump
RUSTC=rustc
LIBCORE=../rust-libcore/target/arm-none-eabihf/release/libcore.rlib
RUSTFLAGS= --target $(TARGET) -C panic=abort -g --crate-type staticlib --extern core=$(LIBCORE)
LINKFLAGS= -O0 -g -Wl,-gc-sections -mfpu=vfp -mfloat-abi=hard -march=armv6zk -mtune=arm1176jzf-s -nostdlib

TARGET=arm-none-eabihf
MAIN=kernel

IMAGE=$(BUILDDIR)/$(MAIN).img
LIST=$(BUILDDIR)/$(MAIN).list
ELF=$(BUILDDIR)/$(MAIN).elf

vpath %.rs $(SRCDIR)
vpath %.o $(OBJDIR)

SOURCES= main.rs 
OBJ= $(addsuffix .o, $(basename $(SOURCES)))

.PHONY: clean 

all: $(IMAGE) $(LIST)

main.o:	main.rs panic.rs

$(IMAGE): $(ELF)
	$(OBJCOPY) $(ELF) -O binary $(IMAGE)

$(LIST): $(IMAGE)
	$(OBJDUMP) -d $(ELF) > $@

$(ELF): $(OBJ) 
	arm-none-eabi-gcc $(LINKFLAGS) -Tsrc/layout.ld  $(addprefix $(OBJDIR)/, $(OBJ)) -o $@

%.o: %.rs 
	$(RUSTC) $(RUSTFLAGS) $< -o $(OBJDIR)/$@

clean:
	rm -f $(OBJDIR)/*
	rm -f $(BUILDDIR)/*
