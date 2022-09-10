#!/bin/bash

# Run this to generate bindings to CMSIS-DSP
# This is only needed when upgrading CMSIS-DSP.

bindgen c/cmsis_dsp_combined.h \
    --use-core --ctypes-prefix crate::ctypes --default-enum-style moduleconsts \
    --whitelist-function "^arm.*" \
    --whitelist-var "^arm.*" \
    --blacklist-type "^__u?int\\d+_t" \
    --output src/bindings.rs  \
    -- -ICMSIS_5-5.9.0/CMSIS/Core/Include -ICMSIS-DSP-1.11.0/Include
