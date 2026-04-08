cmake_minimum_required(VERSION 3.10)
project(AlloyScript VERSION 1.0.0 LANGUAGES C CXX)

set(CMAKE_C_STANDARD 99)
set(CMAKE_CXX_STANDARD 20)

# Task 1.3: Add ALLOY_ENABLE_WEB option
option(ALLOY_ENABLE_WEB "Enable hidden web process and web APIs" ON)

# Task 1.4: Enable ThinLTO
include(CheckIPOSupported)
check_ipo_supported(RESULT LTO_SUPPORTED OUTPUT LTO_ERROR)
if(LTO_SUPPORTED)
    set(CMAKE_INTERPROCEDURAL_OPTIMIZATION TRUE)
endif()

# Task 1.5: Set static linking flags
set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -static-libgcc -static-libstdc++")

include_directories(include include/alloy src/core src/core/backend_adapters packages/microquickjs)

add_library(alloy_core STATIC
    src/core/engine.c
    src/core/stdlib/custom_stdlib.c
    src/core/capability/store.c
    src/core/crypto/crypto.c
    src/core/plugins/config/config_plugin.c
    src/core/plugins/web/web_plugin.c
    src/core/plugins/shell/shell_plugin.c
)

add_executable(alloy src/main.ts) # Placeholder for main entry point
