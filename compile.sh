#!/bin/bash

set -x

make LLVM=1 KDIR=/rfl/obj/linux-x86_64/
gcc -static -o ioctl-userspace ioctl-userspace.c
