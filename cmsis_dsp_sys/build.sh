#!/bin/bash

set -ex

cd ${OUT_DIR}

if ! [ -f CMSIS_5-5.9.0.zip ]; then
    curl -L https://github.com/ARM-software/CMSIS_5/archive/refs/tags/5.9.0.zip -o CMSIS_5-5.9.0.zip
fi

if ! [ -f CMSIS-DSP-1.11.0.zip ]; then
    curl -L https://github.com/ARM-software/CMSIS-DSP/archive/refs/tags/v1.11.0.zip -o CMSIS-DSP-1.11.0.zip
fi

if ! [ -d CMSIS-DSP-1.11.0 ]; then
    unzip CMSIS-DSP-1.11.0.zip
fi
if ! [ -d CMSIS_5-5.9.0 ]; then
    unzip CMSIS_5-5.9.0.zip
fi

cd CMSIS-DSP-1.11.0/Source

if ! [ -f CMakeLists-original.txt ]; then
    cp CMakeLists.txt CMakeLists-original.txt
fi

echo '
set(CMAKE_SYSTEM_NAME Generic)
set(CMAKE_SYSTEM_PROCESSOR arm)
set(CMAKE_TRY_COMPILE_TARGET_TYPE STATIC_LIBRARY)

set(TARGET_TRIPLET "arm-none-eabi-")
set(CMAKE_C_COMPILER   ${TARGET_TRIPLET}gcc)
set(CMAKE_CXX_COMPILER ${TARGET_TRIPLET}g++)
set(CMAKE_ASM_COMPILER ${TARGET_TRIPLET}gcc)
set(CMAKE_LINKER       ${TARGET_TRIPLET}gcc)
set(CMAKE_SIZE_UTIL    ${TARGET_TRIPLET}size)
set(CMAKE_OBJCOPY      ${TARGET_TRIPLET}objcopy)
set(CMAKE_OBJDUMP      ${TARGET_TRIPLET}objdump)
set(CMAKE_NM_UTIL      ${TARGET_TRIPLET}gcc-nm)
set(CMAKE_AR           ${TARGET_TRIPLET}gcc-ar)
set(CMAKE_RANLIB       ${TARGET_TRIPLET}gcc-ranlib)

set(MCPU_FLAGS "-mthumb -mcpu=cortex-m7")
set(VFP_FLAGS "-mfloat-abi=hard -mfpu=fpv5-d16") # Double-precission FPU
set(OPT_FLAGS "-Ofast")
' > CMakeLists.txt
cat CMakeLists-original.txt >> CMakeLists.txt

cmake -DCMSISCORE=../../CMSIS_5-5.9.0/CMSIS/Core .
make clean
make

# See https://github.com/samcrow/cmsis_dsp.rs
cp libCMSISDSP.a ${OUT_DIR}/libarm_cortexM7lfdp_math.a

echo "cargo:rustc-link-search=${OUT_DIR}"
echo "cargo:rustc-link-lib=arm_cortexM7lfdp_math"
