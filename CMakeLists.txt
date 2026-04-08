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

include_directories(
    include
    include/alloy
    src/core
    src/core/backend_adapters
    packages/microquickjs
    packages/webview/core/include
)

add_library(alloy_core STATIC
    src/core/engine.c
    src/core/stdlib/custom_stdlib.c
    src/core/capability/store.c
    src/core/crypto/crypto.c
    src/core/crypto/encrypted_ram.c
    src/core/ipc/broker.c
    src/core/plugins/config/config_plugin.c
    src/core/plugins/web/web_plugin.c
    src/core/plugins/shell/shell_plugin.c
    src/repl/repl_main.c
    packages/mquickjs/readline.c
    packages/mquickjs/dtoa.c
    packages/mquickjs/cutils.c
    packages/mquickjs/libm.c
    packages/mquickjs/mquickjs.c
)

add_executable(alloy src/core/main.c)
target_link_libraries(alloy alloy_core sodium)

add_executable(alloy_test_repl tests/test_repl.c)
target_link_libraries(alloy_test_repl alloy_core sodium)
add_test(NAME repl_tests COMMAND alloy_test_repl)
